//! WebSocket 端点（`GET /ws/{userId}`），对齐 Java `WebSocketEntrypoint`。
//!
//! 与 Java 的**有意偏离**：Java 网关把 `/ws` 挂为 pass-filter（匿名可连，
//! 服务端信任路径中的 userId）。本仓库默认在握手层做鉴权：
//! - token 双通道——`Authorization: Bearer <t>` 头优先，查询参数 `?token=`
//!   兜底（浏览器 WebSocket API 无法自定义请求头，查询参数是浏览器端的
//!   唯一通道）；token 缺失/无效，或为匿名 client_credentials 身份
//!   （id=0）时一律 401；
//! - 路径 userId 必须与 token 主体一致，否则 403（防冒用他人键收听定向
//!   推送）；连接注册键取自认证身份（`vo.id`），不再信任裸路径参数；
//! - `WS_AUTH_REQUIRED=false`（大小写不敏感的 false/0/no）为逃生阀，恢复
//!   Java pass-filter 语义（连接键 = 路径 userId，仅校验键形态），仅建议
//!   在可信内网或迁移期使用。
//!
//! 连接建立后：
//! - 客户端心跳 `{"action":"Ping"}` → 服务端定向回推 `Pong`；
//! - 服务端业务事件（公告/点位/缓存刷新等）经 `_functions::functions::ws`
//!   注册表转发到对应连接。

use axum::{
    extract::{
        Path, Query,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use serde_json::Value;

/// 握手查询参数：`?token=`（浏览器 WebSocket API 无法自定义请求头时的
/// token 通道）。因出现在 `ws_handler` 签名中而 pub（模块本身私有）。
#[derive(Debug, Deserialize)]
pub struct WsHandshakeQuery {
    token: Option<String>,
}

/// `GET /ws/{userId}` 升级握手（鉴权逻辑见 `ws_handshake_key`；Err 即握手
/// 失败，axum 将 StatusCode 转为无 body 的状态码响应）。
pub async fn ws_handler(
    headers: HeaderMap,
    Query(query): Query<WsHandshakeQuery>,
    ws: WebSocketUpgrade,
    Path(user_id): Path<String>,
) -> Result<Response, StatusCode> {
    let key = ws_handshake_key(&headers, query.token, &user_id).await?;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, key)))
}

/// 握手鉴权，返回连接注册键：
/// - 默认（`WS_AUTH_REQUIRED` 未设置或值不为 false/0/no，大小写不敏感）：
///   解析 token（Authorization 头优先、查询参数兜底），失败或匿名身份
///   （id=0）→ 401；路径 userId 与 token 主体不一致 → 403；键 = `vo.id`。
/// - 逃生阀关闭鉴权时：键 = 路径 userId（Java pass-filter 语义），但仍
///   校验键形态（非空、长度 ≤ 64、ASCII 字母数字/`-`/`_`），防无界/怪异
///   键撑爆注册表，不合法 → 400。
///
/// `pub` 供 HTTP 层测试直接断言鉴权契约（#134）：oneshot 请求没有真实
/// 连接升级（`hyper::upgrade::OnUpgrade` 无公开构造器），axum 的
/// `WebSocketUpgrade` 提取器会在进入 handler 前拒绝无升级头的请求，
/// 401/403 的判定因此只能在函数层直测。
pub async fn ws_handshake_key(
    headers: &HeaderMap,
    query_token: Option<String>,
    user_id: &str,
) -> Result<String, StatusCode> {
    let auth_required = std::env::var("WS_AUTH_REQUIRED")
        .map(|v| !matches!(v.trim().to_ascii_lowercase().as_str(), "false" | "0" | "no"))
        .unwrap_or(true);

    if !auth_required {
        let well_formed = !user_id.is_empty()
            && user_id.len() <= 64
            && user_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        return if well_formed {
            Ok(user_id.to_string())
        } else {
            Err(StatusCode::BAD_REQUEST)
        };
    }

    // token 双通道：Authorization: Bearer 头优先，查询参数兜底（浏览器
    // WebSocket API 无法自定义请求头）
    let header_token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);
    let Some(token) = header_token.or(query_token) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let (vo, _claims) = _functions::functions::system::oauth::oauth_parse_token(token.to_string())
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    // 匿名 client_credentials 身份（id=0）不建立推送连接
    if vo.id == 0 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // 路径 userId 必须与 token 主体一致（防冒用他人键收听定向推送）
    let key = vo.id.to_string();
    if user_id != key {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(key)
}

/// 单连接生命周期：注册进连接池，循环处理上行消息与下行事件。
async fn handle_socket(mut socket: WebSocket, user_id: String) {
    // 注册失败（单用户/全局连接上限）：握手已无法撤回，发送 Close 帧后放弃
    // 该连接（socket 随作用域结束 drop）
    let Ok((mut rx, _guard)) = _functions::functions::ws::ws_register(user_id.clone()) else {
        let _ = socket.send(Message::Close(None)).await;
        return;
    };
    let mut shutdown_rx = _functions::functions::ws::shutdown_subscribe();

    loop {
        tokio::select! {
            // 上行：仅处理心跳 Ping（对齐 Java handlerMap）
            incoming = socket.recv() => {
                let Some(Ok(msg)) = incoming else {
                    break;
                };
                match msg {
                    Message::Text(text) => {
                        if let Ok(v) = serde_json::from_str::<Value>(&text)
                            && v.get("action").and_then(Value::as_str) == Some("Ping")
                        {
                            _functions::functions::ws::ws_send_to_users(
                                std::slice::from_ref(&user_id),
                                "Pong",
                                Value::Null,
                            );
                        }
                    },
                    Message::Close(_) => break,
                    _ => {},
                }
            },
            // 下行：业务事件转发到该连接
            event = rx.recv() => {
                let Some(payload) = event else {
                    break;
                };
                if socket.send(Message::Text(Utf8Bytes::from(payload))).await.is_err() {
                    break;
                }
            },
            // 停机广播：main 的优雅停机先广播关闭 WS 会话，再排空 HTTP
            // 在途请求（axum 要等所有连接关闭才退出，长连不关会挂住）。
            // 收到通知即回 Close 帧断开。changed() 只有发送端 drop 才返回
            // Err（全局 static 永不 drop），且值只会 false→true 变化一次，
            // 故任何完成一律按停机处理，取最简写法。
            _ = shutdown_rx.changed() => {
                let _ = socket.send(Message::Close(None)).await;
                break;
            },
        }
    }
}
