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
//! 断开时自动从注册表移除。
//!
//! 与 Java 的**有意偏离**：Java 网关把 `/ws` 挂为 pass-filter（连接本身
//! 不鉴权，服务端信任路径中的 userId）。本仓库在 router 握手层默认要求
//! Bearer token（`WS_AUTH_REQUIRED=false` 逃生阀可恢复 Java 语义，仅建议
//! 迁移期/可信内网使用），注册表键取自认证身份而非裸路径参数；注册时施加
//! 单用户 8 连接与全局 2048 连接上限，事件改走 128 容量的有界通道——
//! 慢消费者队列满即被主动断开（真实背压），杜绝无界队列堆积耗尽内存。
//!
//! 优雅停机配合：`begin_shutdown()` 经全局 watch 通道广播停机，各连接
//! 循环（router 侧 `handle_socket`）经 `shutdown_subscribe()` 订阅，收到
//! 通知即向客户端发送 Close 帧并退出。axum 的 graceful shutdown 要等
//! 所有连接关闭才返回，长连 WebSocket 不主动断开会把进程在停机窗口内
//! 一直挂住，所以 WS 侧必须配合先关。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use tokio::sync::mpsc;

use serde_json::{Value, json};

/// 单连接事件队列容量：慢消费者积压满即被主动断开（见 `ws_broadcast`）。
const WS_CHANNEL_CAPACITY: usize = 128;
/// 单用户连接数上限（同一 userId 的多标签页/多设备会话之和）。
const WS_USER_CONNECTION_LIMIT: usize = 8;
/// 全局连接数上限（注册表内所有用户的连接之和，防连接风暴耗尽内存）。
const WS_GLOBAL_CONNECTION_LIMIT: usize = 2048;

/// 会话注册表：userId → 该用户的全部连接（发送端）。
static WS_SESSIONS: Lazy<Mutex<HashMap<String, Vec<mpsc::Sender<String>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 单个连接的生命周期守卫：drop 时把自身从注册表移除。
pub struct WsGuard {
    user_id: String,
    sender: mpsc::Sender<String>,
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
///
/// 连接上限：单用户 8 连接（`too many connections for this user`）、全局
/// 2048 连接（`too many websocket connections`，遍历注册表求和，连接时
/// O(用户数) 可接受）。超限时返回 Err，调用方应关闭连接。
pub fn ws_register(user_id: String) -> Result<(mpsc::Receiver<String>, WsGuard), &'static str> {
    let (tx, rx) = mpsc::channel(WS_CHANNEL_CAPACITY);
    let mut sessions = WS_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    if sessions
        .get(&user_id)
        .is_some_and(|list| list.len() >= WS_USER_CONNECTION_LIMIT)
    {
        return Err("too many connections for this user");
    }
    if sessions.values().map(Vec::len).sum::<usize>() >= WS_GLOBAL_CONNECTION_LIMIT {
        return Err("too many websocket connections");
    }
    sessions
        .entry(user_id.clone())
        .or_default()
        .push(tx.clone());
    Ok((
        rx,
        WsGuard {
            user_id,
            sender: tx,
        },
    ))
}

/// 全局停机广播：`false` = 正常运行，`true` = 进入优雅停机。
/// 用 watch 是为了让所有已订阅的连接循环同时观察到停机；static 永不
/// drop，发送端在进程存续期间始终可用。
static WS_SHUTDOWN_TX: Lazy<tokio::sync::watch::Sender<bool>> =
    Lazy::new(|| tokio::sync::watch::channel(false).0);

/// 广播 WS 停机：优雅停机开始时由 main 调用一次。用 `send_replace`
/// 而非 `send`——tokio watch 的 `send` 在零接收端（此刻无任何 WS 连接）
/// 时不落值直接返回 Err，`send_replace` 无条件写入，即「更新当前值」
/// 语义；接收端数量为 0 时无人唤醒也无妨（本就没有连接需要关闭）。
pub async fn begin_shutdown() {
    WS_SHUTDOWN_TX.send_replace(true);
}

/// 订阅停机通知（连接循环的 select 分支使用）。
pub fn shutdown_subscribe() -> tokio::sync::watch::Receiver<bool> {
    WS_SHUTDOWN_TX.subscribe()
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

/// 向单个发送端投递事件：`try_send` 而非阻塞 `send`。
/// - `Closed`：对端已断开 → 死连接；
/// - `Full`：慢消费者队列满 → **有意**断开（真实背压）——否则单个不消费的
///   连接会让事件无限积压，拖垮整个进程。
///
/// 两种失败同样计入死连接列表，由 `drop_dead_sessions` 统一清理。
fn try_send_collect_dead(
    dead: &mut Vec<(String, mpsc::Sender<String>)>,
    user_id: &str,
    sender: &mpsc::Sender<String>,
    payload: &str,
) {
    use mpsc::error::TrySendError;
    if let Err(TrySendError::Full(_) | TrySendError::Closed(_)) =
        sender.try_send(payload.to_string())
    {
        dead.push((user_id.to_string(), sender.clone()));
    }
}

/// 清理死连接（断开或积压满的发送端），避免死连接占位导致注册表膨胀。
/// 须在释放注册表锁后调用（重入加锁会死锁）。
fn drop_dead_sessions(dead: Vec<(String, mpsc::Sender<String>)>) {
    if dead.is_empty() {
        return;
    }
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

/// 向所有连接广播事件（Java `broadcast`）。
pub fn ws_broadcast(event: &str, data: Value) {
    let payload = ws_payload(event, data);
    let sessions = WS_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    let mut dead: Vec<(String, mpsc::Sender<String>)> = Vec::new();
    for (user_id, list) in sessions.iter() {
        for s in list {
            try_send_collect_dead(&mut dead, user_id, s, &payload);
        }
    }
    drop(sessions);
    drop_dead_sessions(dead);
}

/// 向指定用户推送事件（Java `sendToUsers`；无此连接时静默忽略）。
/// 与 `ws_broadcast` 相同的 try_send + 死连接清理（此前发送失败被完全忽略）。
pub fn ws_send_to_users(user_ids: &[String], event: &str, data: Value) {
    let payload = ws_payload(event, data);
    let sessions = WS_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    let mut dead: Vec<(String, mpsc::Sender<String>)> = Vec::new();
    for uid in user_ids {
        if let Some(list) = sessions.get(uid) {
            for s in list {
                try_send_collect_dead(&mut dead, uid, s, &payload);
            }
        }
    }
    drop(sessions);
    drop_dead_sessions(dead);
}

/// 防抖推送窗口：对齐 Java `server.cache.debounce-delay` 默认 30s，
/// 写操作触发的缓存清理事件（`*BinaryPurged`）在窗口内合并为一次广播，
/// 避免批量操作（如逐条导入物品）触发事件风暴。
pub const PURGE_DEBOUNCE_WINDOW: Duration = Duration::from_secs(30);

/// 各事件最近一次广播时间（防抖状态）。
static LAST_PUSH: Lazy<Mutex<HashMap<String, Instant>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 带防抖的全员广播：同一事件在窗口内只推一次（窗口内重复调用静默丢弃）。
pub fn ws_broadcast_debounced(event: &str, data: Value, window: Duration) {
    let now = Instant::now();
    {
        let mut last = LAST_PUSH.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(prev) = last.get(event)
            && now.duration_since(*prev) < window
        {
            return;
        }
        last.insert(event.to_string(), now);
    }
    ws_broadcast(event, data);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WS_SESSIONS 是进程级共享静态：串行化各测试，避免并行执行时相互
    /// 看到对方注册的连接（影响「注册表清空」断言）。
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// 注册/广播/定向投递与守卫清理（含「有界通道下正常广播投递不受影响」）。
    #[test]
    fn register_broadcast_and_guard_removal() {
        let _serial = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let (mut rx1, guard1) = ws_register("u1".into()).expect("u1 register");
        let (mut rx2, guard2) = ws_register("u2".into()).expect("u2 register");

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

    /// 单用户连接上限：满 8 个后第 9 个注册被拒；守卫全部释放后可再次注册。
    #[test]
    fn per_user_connection_limit() {
        let _serial = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let mut guards = Vec::new();
        for _ in 0..WS_USER_CONNECTION_LIMIT {
            guards.push(ws_register("lim".into()).expect("within per-user limit").1);
        }
        // WsGuard 未实现 Debug，unwrap_err 不可用，改用 match 断言错误文案
        match ws_register("lim".into()) {
            Err(e) => assert_eq!(
                e, "too many connections for this user",
                "9th connection for the same user must be rejected"
            ),
            Ok(_) => panic!("9th connection for the same user must be rejected"),
        }
        drop(guards);
        assert!(ws_register("lim".into()).is_ok());
    }

    /// 慢消费者背压：不消费事件时广播超过队列容量（128），该连接被主动
    /// 断开并从注册表清理；已入队的事件仍可读（不丢已接受部分）。
    #[test]
    fn slow_consumer_is_disconnected_on_full_queue() {
        let _serial = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let (mut rx, _guard) = ws_register("slow".into()).expect("slow register");
        // 灌入远超容量的广播：第 129 条起触发 Full → 进入死列表并清理
        for i in 0..300 {
            ws_broadcast("Burst", json!(i));
        }
        assert!(
            WS_SESSIONS.lock().unwrap().is_empty(),
            "slow consumer should be dropped from the registry"
        );
        let mut received = 0;
        while rx.try_recv().is_ok() {
            received += 1;
        }
        assert_eq!(received, WS_CHANNEL_CAPACITY);
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
