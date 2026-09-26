//! 通用 IP 限流器：固定窗口计数，窗口锚定在**首次计数**时刻（不随后续
//! 请求滑动）。供公开端点（注册、oauth 发牌/刷新、邀请码查询/消费）共用，
//! 防止未认证接口被批量滥用——尤其每次请求都触发重 CPU 操作的端点
//!（如 `/system/user/register/qq` 的查重 + bcrypt cost-12 ≈ 250ms CPU）。

use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;

use _database::DB_CONN;

/// Redis 不可用/命令失败时的进程内兜底计数表（单实例语义）。
/// key `rl:{bucket}:{ip}` → (计数, 窗口截止时刻)；截止时刻 = 首次计数时刻
/// 加上窗口长度，即「锚定首次计数」的固定窗口（自首次计数起经过
/// `window_secs` 秒后重置）。
static LOCAL_BUCKETS: Lazy<std::sync::Mutex<std::collections::HashMap<String, (u32, i64)>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// 清理窗口已过期的限流条目（防止 `LOCAL_BUCKETS` 按唯一 bucket+IP 无限累积）。
fn sweep_stale_entries(map: &mut std::collections::HashMap<String, (u32, i64)>, now: i64) {
    map.retain(|_, (_, expires_at)| now < *expires_at);
}

/// Redis 计数路径：首击 `SET NX EX` 原子建键（并发首击不会重复建键），
/// 已有键 INCR 累加。返回 `None` 表示 Redis 不可用/命令失败，由调用方
/// 降级到进程内兜底。
async fn count_redis(key: &str, window_secs: u64) -> Option<u32> {
    let client = DB_CONN.wait().redis_conn.as_ref()?;
    let mut conn = client.get_multiplexed_async_connection().await.ok()?;
    let created: bool = redis::cmd("SET")
        .arg(key)
        .arg(1)
        .arg("NX")
        .arg("EX")
        .arg(window_secs)
        .query_async(&mut conn)
        .await
        .ok()?;
    let count: u32 = if created {
        1
    } else {
        let n: i64 = redis::cmd("INCR")
            .arg(key)
            .query_async(&mut conn)
            .await
            .ok()?;
        // 仅当 INCR 返回 1（键在 SET 与 INCR 之间恰好过期、被 INCR 重建为无
        // TTL 的键）时补一次 EXPIRE，防止计数键永不过期；其余情况不碰 TTL
        // ——无条件续期会把固定窗口劣化为随每次请求滑动的滚动窗口。
        if n == 1 {
            let _: i64 = redis::cmd("EXPIRE")
                .arg(key)
                .arg(window_secs)
                .query_async(&mut conn)
                .await
                .ok()?;
        }
        n.max(0) as u32
    };
    Some(count)
}

/// 进程内兜底计数（Redis 不可用时的降级路径）：计数 +1 并返回计数值。
fn count_local(key: &str, window_secs: u64) -> u32 {
    let now = chrono::Utc::now().timestamp();
    let mut map = LOCAL_BUCKETS.lock().unwrap();
    sweep_stale_entries(&mut map, now);
    let entry = map
        .entry(key.to_string())
        .or_insert((0, now.saturating_add(window_secs as i64)));
    // 窗口已过（自首次计数起超过 window_secs）：重置并重新锚定到当前时刻
    if now >= entry.1 {
        *entry = (0, now.saturating_add(window_secs as i64));
    }
    entry.0 += 1;
    entry.0
}

/// 通用 IP 限流入口：`bucket` 区分端点，语义为「每 window_secs 秒每 IP
/// 最多 limit 次请求」，计数值超过 limit 时拒绝。放在业务逻辑最前面，
/// 让滥用请求在触碰任何 DB / bcrypt 工作前就被挡掉。
///
/// 计数优先走 Redis（key `rl:{bucket}:{ip}`）：多实例部署共享同一计数，
/// 攻击者无法靠打 N 个副本把额度放大 N 倍。Redis 不可用/命令失败时降级
/// 为进程内 HashMap（保留单实例行为，避免 Redis 抖动导致全站拒绝服务）。
pub async fn enforce_ip_rate_limit(
    bucket: &str,
    ip: SocketAddr,
    limit: u32,
    window_secs: u64,
) -> Result<()> {
    // key 只取 IP（不含端口）：同一客户端的临时端口随连接变化，按完整
    // SocketAddr 分桶等于不限流。
    let key = format!("rl:{}:{}", bucket, ip.ip());
    let count = match count_redis(&key, window_secs).await {
        Some(count) => count,
        None => count_local(&key, window_secs),
    };
    if count > limit {
        return Err(anyhow!("Too many requests; please try again later"));
    }
    Ok(())
}
