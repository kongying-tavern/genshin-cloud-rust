//! WebSocket 端点（`GET /ws/{userId}`），对齐 Java `WebSocketEntrypoint`。
//!
//! 连接建立后：
//! - 客户端心跳 `{"action":"Ping"}` → 服务端定向回推 `Pong`；
//! - 服务端业务事件（公告/点位/缓存刷新等）经 `_functions::functions::ws`
//!   注册表转发到对应连接。

use axum::{
    extract::{
        Path,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde_json::Value;

/// `GET /ws/{userId}` 升级握手。
pub async fn ws_handler(ws: WebSocketUpgrade, Path(user_id): Path<String>) -> Response {
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
