//! Socket.IO 兼容推送通道（`socketioxide`）。
//!
//! Java 上游与最新前端（`map_register_v3`，`socket.io-client` 4.x）的推送
//! 协议都是 **Socket.IO**——本模块让 Rust 后端与生态对齐：前端照常以
//! `io(VITE_WS_BASE, { path, query: { userId } })` 连接，收到的事件仍是
//! `{event, message, data, time}` 信封（事件名统一为 `message`，载荷为
//! 信封的 JSON **字符串**——前端 `socket.on("message", s => JSON.parse(s))`）。
//!
//! 与裸 WS（`/ws/{userId}`，保留作过渡）的复用关系：
//! - 鉴权复用 `crate::routes::ws::ws_handshake_key`（token 双通道 /
//!   `WS_AUTH_REQUIRED` 逃生阀 / 主体一致性 / userId 形态校验，规则相同）；
//! - 事件投递复用 `functions::ws::ws_register` 注册表——广播、定向、上限
//!   与背压对两种通道**同一份实现**（单用户 8 连接 / 全局 2048 / 128 容量）；
//! - 心跳：socket.io 协议自带 engine.io 心跳（socketioxide 处理），应用层
//!   另实现 Java 的 `rttcheck` 回环（带时间戳回显）；
//! - 优雅停机：复用 `functions::ws::shutdown_subscribe`，收到停机即断开。
//!
//! 握手路径经 `WS_SOCKETIO_PATH` 配置（默认 `/socket.io`，即 socket.io
//! 生态默认值；前端 `VITE_WS_BASE` 需指向同一路径）。独立路径使其与保留
//! 的裸 WS `/ws/{userId}` 共存（socket.io-client 会把 path 当 engine.io
//! 端点，请求形如 `/socket.io/?EIO=4&transport=websocket`）。

use std::collections::HashMap;

use anyhow::Result;
use socketioxide::SocketIo;

use crate::routes::{SharedDb, ws::ws_handshake_key};
use _functions::functions::ws;

/// 挂载 Socket.IO 命名空间并返回可应用到 Router 的层。
///
/// 握手路径经 `WS_SOCKETIO_PATH` 配置（默认 `/socket.io`）。
pub fn mount(db: SharedDb) -> Result<socketioxide::layer::SocketIoLayer> {
    let path = std::env::var("WS_SOCKETIO_PATH").unwrap_or_else(|_| "/socket.io".into());
    let (layer, io) = socketioxide::SocketIo::builder()
        .req_path(path)
        .build_layer();
    register_namespace(io, db);
    Ok(layer)
}

/// 在 `SocketIo` 实例上注册 `/` 命名空间的连接处理器。
fn register_namespace(io: SocketIo, db: SharedDb) {
    io.ns("/", move |s: socketioxide::extract::SocketRef| {
        let db = db.clone();
        async move { handle_connect(s, db).await }
    });
}

/// 连接握手：鉴权（复用裸 WS 的 `ws_handshake_key`，规则完全一致）→
/// 注册表注册 → 事件转发任务 + `rttcheck` 回环。失败一律断开。
async fn handle_connect(s: socketioxide::extract::SocketRef, db: SharedDb) {
    // rttcheck 回环（Java SocketIOConfiguration 同名事件）：立即回显时间戳，
    // 前端据此计算 RTT。Java 侧带 debounce；回显本身开销极小，这里直接回。
    s.on(
        "rttcheck",
        |s: socketioxide::extract::SocketRef,
         socketioxide::extract::Data::<serde_json::Value>(data)| async move {
            let id = data.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let now = chrono::Utc::now().timestamp_millis();
            let _ = s.emit(
                "rttcheck",
                &serde_json::json!({
                    "id": id,
                    "receiveTimestamp": now - 1,
                    "sendTimestamp": now,
                }),
            );
        },
    );
    // socket.io-client 把自定义 query 放在 engine.io 握手 URL 上
    // （`?userId=..&token=..`），因此 query 与 Authorization 头两条 token
    // 通道都与裸 WS 一致可用。
    let (query, headers) = {
        let parts = s.req_parts();
        (
            parts.uri.query().unwrap_or("").to_string(),
            parts.headers.clone(),
        )
    };
    let params = parse_query(&query);
    let user_id = params.get("userId").cloned().unwrap_or_default();
    let token = params.get("token").cloned();

    // 鉴权规则与裸 WS 完全一致：成功得注册键（= 认证主体），失败即断开。
    if let Err(status) = ws_handshake_key(db.as_ref(), &headers, token, &user_id).await {
        tracing::warn!("socket.io handshake rejected (user_id={user_id:?}, status={status})");
        let _ = s.disconnect();
        return;
    }

    // 注册进共享注册表：广播 / 定向 / 上限 / 背压与裸 WS 同一份实现。
    let Ok((mut rx, guard)) = ws::ws_register(user_id) else {
        let _ = s.disconnect();
        return;
    };
    // 事件转发：注册表通道里的每条信封（已序列化的 JSON 文本）以事件名
    // `message`、**字符串载荷**推给客户端（前端 JSON.parse 还原信封）。
    tokio::spawn(async move {
        let _guard = guard;
        let mut shutdown = ws::shutdown_subscribe();
        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_ok() && *shutdown.borrow() {
                        let _ = s.disconnect();
                        break;
                    }
                },
                msg = rx.recv() => match msg {
                    // 客户端已断开时 emit 返回 Err → 退出并释放注册表。
                    Some(payload) => {
                        if s.emit("message", &payload).is_err() {
                            break;
                        }
                    },
                    None => {
                        let _ = s.disconnect();
                        break;
                    },
                },
            }
        }
    });
}

/// 极简 query 解析（`k=v&k=v`，值做百分号解码）。JWT 是 base64url + 点、
/// userId 是字母数字——都不含需转义字符，与 axum `Query` 的解码语义一致。
fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = urlencoding::decode(it.next().unwrap_or(""))
            .unwrap_or_default()
            .into_owned();
        let v = urlencoding::decode(it.next().unwrap_or(""))
            .unwrap_or_default()
            .into_owned();
        if !k.is_empty() {
            map.insert(k, v);
        }
    }
    map
}
