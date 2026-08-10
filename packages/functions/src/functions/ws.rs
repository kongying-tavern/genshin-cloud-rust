//! WebSocket 连接注册表与事件推送。
//!
//! 对齐 Java 参考实现 `WebSocketEntrypoint`（`/ws/{userId}`）：
//! - 服务端接收 `{"action":"Ping"}`，回推 `{"event":"Pong",...}` 心跳；
//! - 业务事件统一由服务端推送 `{event, message, data, time}` 结构
//!   （`W<T>`），推送通道与 Java 一致：
//!   - `ws_broadcast` —— 全员广播（`broadcast`）；
//!   - `ws_send_to_users` —— 定向发送（`sendToUsers`）。
//!
//! 连接按 userId 分组管理（同一用户多标签页/多设备各自独立会话），
//! 断开时自动从注册表移除。与 Java 一致，连接本身不做鉴权
//! （前端仅在已登录时发起连接，服务端信任路径中的 userId）。

use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use tokio::sync::mpsc;

use serde_json::{Value, json};

/// 会话注册表：userId → 该用户的全部连接（发送端）。
static WS_SESSIONS: Lazy<Mutex<HashMap<String, Vec<mpsc::UnboundedSender<String>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 单个连接的生命周期守卫：drop 时把自身从注册表移除。
pub struct WsGuard {
    user_id: String,
    sender: mpsc::UnboundedSender<String>,
}

impl Drop for WsGuard {
    fn drop(&mut self) {
        let mut sessions = match WS_SESSIONS.lock() {
            Ok(s) => s,
            Err(_) => return,
        };
        if let Some(list) = sessions.get_mut(&self.user_id) {
            list.retain(|s| !s.same_channel(&self.sender));
            if list.is_empty() {
                sessions.remove(&self.user_id);
            }
        }
    }
}

/// 注册一个连接（返回事件接收端与守卫；守卫释放即断开）。
pub fn ws_register(user_id: String) -> (mpsc::UnboundedReceiver<String>, WsGuard) {
    let (tx, rx) = mpsc::unbounded_channel();
    let mut sessions = WS_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    sessions
        .entry(user_id.clone())
        .or_default()
        .push(tx.clone());
    (
        rx,
        WsGuard {
            user_id,
            sender: tx,
        },
    )
}

/// 构造 Java `W<T>` 结构的 JSON 文本：`{event, message, data, time}`。
fn ws_payload(event: &str, data: Value) -> String {
    let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    json!({
        "event": event,
        "message": "",
        "data": data,
        "time": time,
    })
    .to_string()
}

/// 向所有连接广播事件（Java `broadcast`）。
pub fn ws_broadcast(event: &str, data: Value) {
    let payload = ws_payload(event, data);
    let sessions = WS_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    let mut dead: Vec<(String, mpsc::UnboundedSender<String>)> = Vec::new();
    for (user_id, list) in sessions.iter() {
        for s in list {
            if s.send(payload.clone()).is_err() {
                dead.push((user_id.clone(), s.clone()));
            }
        }
    }
    // 清理已断开的发送端（避免死连接占位导致注册表膨胀）
    if !dead.is_empty() {
        drop(sessions);
        if let Ok(mut sessions) = WS_SESSIONS.lock() {
            for (user_id, sender) in dead {
                if let Some(list) = sessions.get_mut(&user_id) {
                    list.retain(|s| !s.same_channel(&sender));
                    if list.is_empty() {
                        sessions.remove(&user_id);
                    }
                }
            }
        }
    }
}

/// 向指定用户推送事件（Java `sendToUsers`；无此连接时静默忽略）。
pub fn ws_send_to_users(user_ids: &[String], event: &str, data: Value) {
    let payload = ws_payload(event, data);
    let sessions = WS_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    for uid in user_ids {
        if let Some(list) = sessions.get(uid) {
            for s in list {
                let _ = s.send(payload.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_broadcast_and_guard_removal() {
        let (mut rx1, guard1) = ws_register("u1".into());
        let (mut rx2, guard2) = ws_register("u2".into());

        ws_broadcast("NoticeAdded", json!(42));
        assert_eq!(
            rx1.try_recv().expect("u1 should receive"),
            ws_payload("NoticeAdded", json!(42))
        );
        assert_eq!(
            rx2.try_recv().expect("u2 should receive"),
            ws_payload("NoticeAdded", json!(42))
        );

        ws_send_to_users(&["u1".into()], "UserKickedOut", Value::Null);
        assert_eq!(
            rx1.try_recv().expect("u1 should receive kick"),
            ws_payload("UserKickedOut", Value::Null)
        );
        assert!(rx2.try_recv().is_err(), "u2 should not receive kick");

        drop(guard1);
        drop(guard2);
        // 注册表清空后广播不再投递
        ws_broadcast("AppUpdated", Value::Null);
        assert!(rx1.try_recv().is_err());
        assert!(rx2.try_recv().is_err());
        assert!(WS_SESSIONS.lock().unwrap().is_empty());
    }

    #[test]
    fn payload_matches_java_w_shape() {
        let raw = ws_payload("MarkerAdded", json!(7));
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["event"], "MarkerAdded");
        assert_eq!(v["data"], 7);
        assert_eq!(v["message"], "");
        assert!(v["time"].as_str().unwrap().len() >= 19);
    }
}
