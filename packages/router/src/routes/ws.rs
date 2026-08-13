//! WebSocket 端点（`GET /ws/{userId}`），对齐 Java `WebSocketEntrypoint`。
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
    response::Response,
};
use serde_json::Value;

/// `GET /ws/{userId}` 升级握手。
#[utoipa::path(
    get,
    path = "/ws/{user_id}",
    tag = "ws",
    summary = "WebSocket 连接（心跳 Ping/Pong 与业务事件推送）",
    params(("user_id" = String, Path, description = "用户 ID")),
    responses(
        (status = 101, description = "WebSocket 升级成功"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
pub async fn ws_handler(ws: WebSocketUpgrade, Path(user_id): Path<String>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, user_id))
}

/// `GET /ws?userId={id}` 升级握手（前端契约：userId 走 query）。
#[derive(serde::Deserialize, Default)]
pub struct WsQuery {
    #[serde(rename = "userId", default)]
    pub user_id: Option<String>,
}

/// 兼容前端 wss://.../ws?userId=xxx 形式的连接。
#[utoipa::path(
    get,
    path = "/ws",
    tag = "ws",
    summary = "WebSocket 连接（userId 走 query 的兼容入口）",
    params(("userId" = Option<String>, Query, description = "用户 ID")),
    responses(
        (status = 101, description = "WebSocket 升级成功"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
pub async fn ws_handler_query(ws: WebSocketUpgrade, Query(q): Query<WsQuery>) -> Response {
    let user_id = q.user_id.unwrap_or_default();
    ws.on_upgrade(move |socket| handle_socket(socket, user_id))
}

/// 单连接生命周期：注册进连接池，循环处理上行消息与下行事件。
async fn handle_socket(mut socket: WebSocket, user_id: String) {
    let (mut rx, _guard) = _functions::functions::ws::ws_register(user_id.clone());

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
        }
    }
}
