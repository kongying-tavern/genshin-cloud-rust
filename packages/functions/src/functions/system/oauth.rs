use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use std::net::SocketAddr;

use redis::{AsyncTypedCommands, SetOptions};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryOrder,
    prelude::*,
};

use _database::{DB_CONN, models};
use _utils::{
    bcrypt::verify_password,
    db_operations::SafeEntityTrait,
    jwt::{Claims, EXPIRED_APPEND_DURATION, REFRESH_APPEND_DURATION, generate_token, verify_token},
    models::SysUserVO,
    types::{
        AccessPolicyItemEnum, AccessPolicyList, SystemActionLogAction, SystemUserRole,
        auth::{OauthAnonymousResponse, OauthLoginResponse, OauthScopeType, OauthTokenType},
    },
};

/// 基础设施错误（数据库 / Redis 等）不回传实现细节，统一为通用文案，
/// 避免向客户端暴露 SQL / 连接等内部信息。
fn internal_error(_err: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("Internal server error")
}

/// 按用户的 access_policy 校验登录环境（IP / 设备）。
///
/// 数据源为 `sys_user_device` 表：
/// - `ip:same_last_ip`：请求 IP 必须等于该用户最近一次登录的 IP
/// - `dev:same_last_device`：请求 User-Agent 必须等于最近一次登录的设备
/// - `ip:block_disallow_ip` / `dev:block_disallow_device`：命中 `status != 0`
///   （禁用）的登记条目时拒绝
/// - 其余策略（允许列表、地区等）在当前数据模型下无对应存储，放行
/// - 无任何历史记录（首次登录）时，same_* 类策略放行
async fn check_access_policy(
    user_id: i64,
    access_policy: &[AccessPolicyItemEnum],
    ip: SocketAddr,
    user_agent: &str,
) -> Result<()> {
    // 本地/测试环境可跳过全部登录环境策略（IP/设备绑定等）。
    if std::env::var("SKIP_ACCESS_POLICY").as_deref() == Ok("true") {
        return Ok(());
    }
    let db = &DB_CONN.wait().pg_conn;
    let last = models::system::sys_user_device::Entity::find_safety()
        .filter(models::system::sys_user_device::Column::UserId.eq(Some(user_id)))
        .order_by_desc(models::system::sys_user_device::Column::LastLoginTime)
        .one(db)
        .await
        .map_err(internal_error)?;

    for policy in access_policy {
        match policy {
            AccessPolicyItemEnum::IpSameLastIp => {
                if let Some(dev) = &last
                    && let Some(last_ip) = &dev.ipv4
                    && ip.ip().to_string() != *last_ip
                {
                    return Err(anyhow!(
                        "Access denied: IP {} does not match the last login IP {last_ip}",
                        ip
                    ));
                }
            },
            AccessPolicyItemEnum::DevSameLastDevice => {
                if let Some(dev) = &last
                    && dev.device_id != *user_agent
                {
                    return Err(anyhow!(
                        "Access denied: device does not match the last login device"
                    ));
                }
            },
            AccessPolicyItemEnum::IpBlockDisallowIp => {
                let blocked = models::system::sys_user_device::Entity::find_safety()
                    .filter(models::system::sys_user_device::Column::UserId.eq(Some(user_id)))
                    .filter(models::system::sys_user_device::Column::Status.ne(0))
                    .filter(
                        models::system::sys_user_device::Column::Ipv4.eq(Some(ip.ip().to_string())),
                    )
                    .one(db)
                    .await
                    .map_err(internal_error)?;
                if blocked.is_some() {
                    return Err(anyhow!("Access denied: IP {} is blocked", ip));
                }
            },
            AccessPolicyItemEnum::DevBlockDisallowDevice => {
                let blocked = models::system::sys_user_device::Entity::find_safety()
                    .filter(models::system::sys_user_device::Column::UserId.eq(Some(user_id)))
                    .filter(models::system::sys_user_device::Column::Status.ne(0))
                    .filter(models::system::sys_user_device::Column::DeviceId.eq(user_agent))
                    .one(db)
                    .await
                    .map_err(internal_error)?;
                if blocked.is_some() {
                    return Err(anyhow!("Access denied: device is blocked"));
                }
            },
            // 允许列表 / 地区类策略：当前数据模型无对应存储，放行
            _ => {},
        }
    }
    Ok(())
}

/// 登录成功后登记设备（upsert `sys_user_device`：user_id + device_id 唯一）。
async fn record_device(user_id: i64, ip: SocketAddr, user_agent: &str) -> Result<()> {
    let db = &DB_CONN.wait().pg_conn;
    let existing = models::system::sys_user_device::Entity::find_safety()
        .filter(models::system::sys_user_device::Column::UserId.eq(Some(user_id)))
        .filter(models::system::sys_user_device::Column::DeviceId.eq(user_agent))
        .one(db)
        .await?;

    let now = chrono::Utc::now().naive_utc();
    if let Some(dev) = existing {
        // 已有登记：仅刷新 IP / 最近登录时间。status（封禁状态）随
        // `dev.into()` 原样保留，不因成功登录被静默重置为 0。
        let mut am: models::system::sys_user_device::ActiveModel = dev.into();
        // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
        am.updater_id = Set(Some(user_id));
        am.ipv4 = Set(Some(ip.ip().to_string()));
        am.last_login_time = Set(Some(now));
        models::system::sys_user_device::Entity::update_safety(am)?
            .exec(db)
            .await?;
        return Ok(());
    }

    // 首次登记：status 初始化为 0（正常）。
    let am = models::system::sys_user_device::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(user_id)),
        updater_id: Set(Some(user_id)),
        del_flag: Set(false),
        user_id: Set(Some(user_id)),
        device_id: Set(user_agent.to_string()),
        ipv4: Set(Some(ip.ip().to_string())),
        status: Set(0),
        last_login_time: Set(Some(now)),
    };
    // 并发首次登录同设备时，两次 check 都可能查不到（check-then-act
    // 竞态）；数据库唯一约束冲突（Postgres SQLSTATE 23505）由后插者
    // 触发，视为已登记成功，直接忽略。
    match models::system::sys_user_device::Entity::insert(am)
        .exec(db)
        .await
    {
        Ok(_) => Ok(()),
        Err(DbErr::Exec(RuntimeErr::SqlxError(err)))
            if matches!(
                err.as_ref(),
                sea_orm::sqlx::Error::Database(db_err)
                    if db_err.code().as_deref() == Some("23505")
            ) =>
        {
            Ok(())
        },
        Err(err) => Err(err.into()),
    }
}

/// 登录响应 `env` 字段来源（Java 契约：Spring active profile 的对应物）。
/// 读 `APP_ENV`（去空白、转小写），未设置或为空时默认 `dev`；生产部署应
/// 显式设置 `APP_ENV=prod`。
fn environment_id() -> String {
    std::env::var("APP_ENV")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "dev".to_string())
}

/// 为用户签发 access/refresh token（含 Redis 会话存储，Redis 不可用时降级）。
///
/// access 与 refresh 使用**不同的 jti**（S2：access token 无法冒充 refresh），
/// Redis 键结构：
/// - `jwt:access:{uid}:{access_jti}` → 用户 VO JSON（受保护端点校验）
/// - `jwt:refresh:{uid}:{refresh_jti}` → 配对 access_jti 字符串（refresh
///   端点校验；轮换时据此吊销旧 access token）
async fn issue_token(item: &models::system::sys_user::Model) -> Result<OauthLoginResponse> {
    let access_jti = Uuid::now_v7();
    let refresh_jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(
        now,
        item.id,
        access_jti,
        "access",
        chrono::Duration::days(15),
    )
    .await?;
    let refresh_token = generate_token(
        now,
        item.id,
        refresh_jti,
        "refresh",
        chrono::Duration::days(30),
    )
    .await?;

    let id = item.id;
    let vo: SysUserVO = item.clone().into();

    // Store token in Redis if available (graceful degradation for e2e mode
    // where Redis is not running — oauth_parse_token will fall back to
    // JWT + DB validation without Redis session lookup).
    if let Some(redis_client) = &DB_CONN.wait().redis_conn
        && let Ok(mut redis_conn) = redis_client.get_multiplexed_async_connection().await
    {
        let _ = redis_conn
            .set_options(
                format!("jwt:access:{}:{}", id, access_jti),
                serde_json::to_string(&vo)?,
                SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::EX(
                        EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                    )),
            )
            .await;
        let _ = redis_conn
            .set_options(
                format!("jwt:refresh:{}:{}", id, refresh_jti),
                access_jti.to_string(),
                SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::EX(
                        REFRESH_APPEND_DURATION.as_seconds_f32() as u64,
                    )),
            )
            .await;
    }

    Ok(OauthLoginResponse {
        access_token,
        refresh_token,
        token_type: OauthTokenType::Bearer,
        expires_in: EXPIRED_APPEND_DURATION.as_seconds_f32() as i64,
        scope: OauthScopeType::All,
        jti: access_jti,
        // Java 契约（AuthorizationServerConfiguration additionalInfo）：
        // 前端 `SysToken` 依赖 userId / userRoles 恢复用户态与权限掩码；
        // env / message 为必有字符串字段（null 会破坏前端契约）。
        user_id: id,
        user_roles: vec![role_code(item.role_id)],
        env: Some(environment_id()),
        message: Some(String::new()),
    })
}

/// SystemUserRole → Java `RoleEnum` code（前端 `RoleTypeEnum` 契约）。
/// 契约代码保持 `MAP_NEIGUI`（前端按此字符串匹配），勿改。
fn role_code(role: _utils::types::SystemUserRole) -> String {
    match role {
        _utils::types::SystemUserRole::Admin => "ADMIN",
        _utils::types::SystemUserRole::MapManager => "MAP_MANAGER",
        _utils::types::SystemUserRole::MapBeta => "MAP_NEIGUI",
        _utils::types::SystemUserRole::MapPunctuate => "MAP_PUNCTUATE",
        _utils::types::SystemUserRole::MapUser => "MAP_USER",
        _utils::types::SystemUserRole::Visitor => "VISITOR",
    }
    .to_string()
}

async fn oauth_password_login_inner(
    item: models::system::sys_user::Model,
    password_raw: String,
    ip: SocketAddr,
    user_agent: &str,
) -> Result<OauthLoginResponse> {
    if !verify_password(password_raw, item.password.clone())? {
        return Err(anyhow!("Invalid username or password"));
    }

    // 身份验证通过后，按用户的 access_policy 校验登录环境。
    // Java `AuthorizationServerConfiguration.checkDeviceAccess`：策略不命中只
    // 影响提示信息与审计日志，登录仍然成功 —— 对齐为「警告放行」而非拒绝。
    let policy: Vec<_> = item.access_policy.clone().map(|a| a.0).unwrap_or_default();
    if let Err(e) = check_access_policy(item.id, &policy, ip, user_agent).await {
        tracing::warn!(user_id = item.id, error = %e, "access policy violation (allowed through, Java-compatible)");
    }

    issue_token(&item).await
}

/// 匿名（client_credentials）身份 VO：id=0 + VISITOR 角色。
///
/// `AuthInfo::is_anonymous` / `require_non_anonymous`（jwt.rs）只按
/// `info.id == 0` 判定，因此构造 id=0 即自动实现：只读接口放行、
/// 写操作（require_non_anonymous）拒绝。
fn anonymous_vo() -> SysUserVO {
    SysUserVO {
        id: 0,
        username: "anonymous".into(),
        nickname: None,
        qq: None,
        phone: None,
        logo: None,
        role_id: SystemUserRole::Visitor,
        access_policy: AccessPolicyList(vec![]),
        remark: None,
    }
}

pub async fn oauth_parse_token(token: String) -> Result<(SysUserVO, Claims)> {
    let claims = verify_token(&token).await?;

    // S2：refresh token 不得当 access token 使用。旧令牌无 token_type
    // 声明（None），走下方 access key 校验路径，保持兼容。
    if claims.token_type.as_deref() == Some("refresh") {
        return Err(anyhow!("Refresh token cannot be used as an access token"));
    }

    // S1：会话校验。Redis key 命中 → 放行；key 明确不存在 → 会话已被吊销
    // （登出/踢出/改密/刷新轮换删除），必须拒绝；仅当 Redis 连接/命令失败
    // （无法判定存在性）时降级 DB 校验。REDIS_REQUIRED=true 时 Redis 不可
    // 用直接失败（fail-closed），不降级。
    let redis_required = std::env::var("REDIS_REQUIRED").as_deref() == Ok("true");
    if let Some(redis_client) = &DB_CONN.wait().redis_conn {
        let key = format!("jwt:access:{}:{}", claims.sub, claims.jti);
        match redis_client.get_multiplexed_async_connection().await {
            Ok(mut redis_conn) => match redis_conn.get::<String>(key).await {
                // 会话命中：以 Redis 中缓存的 VO 为准
                Ok(Some(item)) => return parse_cached_payload(&item, &claims),
                // 键明确不存在 = 吊销已生效（登出/踢出/改密/刷新轮换）
                Ok(None) => return Err(anyhow!("Token revoked")),
                // Redis 命令失败（连接中断等）→ 无法判定存在性
                Err(_) if redis_required => {
                    return Err(anyhow!("Redis unavailable (REDIS_REQUIRED=true)"));
                },
                Err(_) => {},
            },
            // Redis 连接失败 → 无法判定存在性
            Err(_) if redis_required => {
                return Err(anyhow!("Redis unavailable (REDIS_REQUIRED=true)"));
            },
            Err(_) => {},
        }
    }

    // 匿名身份（sub=0）不需要也不能查库（库中无 id=0 用户），直接构造匿名 VO。
    if claims.sub == 0 {
        return Ok((anonymous_vo(), claims));
    }

    // Fallback: look up user from DB by the JWT subject (user_id)
    let user = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::Id.eq(claims.sub))
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .one(&DB_CONN.wait().pg_conn)
        .await
        .map_err(internal_error)?
        .ok_or(anyhow!("User not found for token"))?;
    Ok((user.into(), claims))
}

/// 解析 Redis 中缓存的 access payload：`{"anon":true}`（client_credentials
/// 发牌）→ 构造匿名 VO；否则按 `SysUserVO` 反序列化用户 VO。
fn parse_cached_payload(item: &str, claims: &Claims) -> Result<(SysUserVO, Claims)> {
    let value: serde_json::Value = serde_json::from_str(item)?;
    if value.get("anon").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok((anonymous_vo(), claims.clone()));
    }
    Ok((serde_json::from_value::<SysUserVO>(value)?, claims.clone()))
}

/// 登录暴力破解限流：按 IP 固定窗口（每分钟最多 5 次失败的密码登录尝试）。
/// 只计数失败尝试（成功登录不消耗额度），窗口过后自动重置。
///
/// 计数优先走 Redis（key `login_fail:{ip}`，60s 窗口，NX+EXPIRE+INCR 模式）：
/// 多实例部署共享同一计数，攻击者无法靠打 N 个副本把额度放大 N 倍。Redis
/// 不可用/命令失败时降级为进程内 HashMap（保留原有单实例行为，避免 Redis
/// 抖动导致全站拒绝登录）。
const LOGIN_RATE_LIMIT_PER_MINUTE: u32 = 5;
const LOGIN_RATE_LIMIT_WINDOW_SECS: u64 = 60;

static LOGIN_FAILURES: Lazy<std::sync::Mutex<std::collections::HashMap<String, (u32, i64)>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// 清理窗口已过期的限流条目（防止 `LOGIN_FAILURES` 按唯一 IP 无限累积）。
fn sweep_stale_login_failures(map: &mut std::collections::HashMap<String, (u32, i64)>, now: i64) {
    map.retain(|_, (_, window_start)| {
        now.saturating_sub(*window_start) < LOGIN_RATE_LIMIT_WINDOW_SECS as i64
    });
}

/// 进程内兜底的滚动窗口语义：自**首次失败**起 60 秒内持续计数，与 Redis
/// 侧 `SET NX EX 60` + INCR 的滚动 TTL 对齐。此前的「时钟分钟桶」实现
///（`now / 60` 分桶）会在分钟边界把计数清零——5 次失败跨过整分后第 6 次
/// 放行（CI 无 Redis 走本路径时为确定性抖动源）。
///
/// Redis 侧失败计数。返回 `None` 表示 Redis 不可用/命令失败，由调用方降级
/// 到进程内 HashMap。
///
/// `record=true` 用 NX+EXPIRE+INCR 模式写入：`SET key 1 NX EX 60` 原子创建
/// 窗口（并发首击不会重复建键/续期），已有键则 INCR 累加；INCR 前键恰好
/// 过期时会新建无 TTL 的键，补设一次过期保证窗口仍为 60s。
async fn login_failure_count_redis(ip: SocketAddr, record: bool) -> Option<u32> {
    let client = DB_CONN.wait().redis_conn.as_ref()?;
    let mut conn = client.get_multiplexed_async_connection().await.ok()?;
    // 关键 key 只取 IP（不含端口）
    let key = format!("login_fail:{}", ip.ip());
    if record {
        let created: bool = redis::cmd("SET")
            .arg(&key)
            .arg(1)
            .arg("NX")
            .arg("EX")
            .arg(LOGIN_RATE_LIMIT_WINDOW_SECS)
            .query_async(&mut conn)
            .await
            .ok()?;
        let count: u32 = if created {
            1
        } else {
            let n: i64 = redis::cmd("INCR")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .ok()?;
            let _: i64 = redis::cmd("EXPIRE")
                .arg(&key)
                .arg(LOGIN_RATE_LIMIT_WINDOW_SECS)
                .query_async(&mut conn)
                .await
                .ok()?;
            n.max(0) as u32
        };
        Some(count)
    } else {
        let count: i64 = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()?;
        Some(count.max(0) as u32)
    }
}

/// 限流检查主入口：Redis 可用时以 Redis 计数为准（跨实例一致），Redis 不可
/// 用时降级到进程内 map（保留原单实例行为）。
async fn check_login_rate_limit(ip: SocketAddr) -> Result<()> {
    match login_failure_count_redis(ip, false).await {
        Some(count) if count >= LOGIN_RATE_LIMIT_PER_MINUTE => Err(anyhow!(
            "Too many failed login attempts; try again in a minute"
        )),
        Some(_) => Ok(()),
        None => check_login_rate_limit_local(ip),
    }
}

/// 进程内 HashMap 兜底实现（Redis 不可用时的降级路径）。
fn check_login_rate_limit_local(ip: SocketAddr) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let mut map = LOGIN_FAILURES.lock().unwrap();
    sweep_stale_login_failures(&mut map, now);
    // 同 login_failure_count_redis：key 仅取 IP 不含临时端口
    let entry = map.entry(ip.ip().to_string()).or_insert((0, now));
    if now.saturating_sub(entry.1) >= LOGIN_RATE_LIMIT_WINDOW_SECS as i64 {
        *entry = (0, now);
    }
    if entry.0 >= LOGIN_RATE_LIMIT_PER_MINUTE {
        return Err(anyhow!(
            "Too many failed login attempts; try again in a minute"
        ));
    }
    Ok(())
}

/// 失败计数入口：Redis 可用时写 Redis，不可用时降级到进程内 map。
async fn record_login_failure(ip: SocketAddr) {
    if login_failure_count_redis(ip, true).await.is_some() {
        return;
    }
    record_login_failure_local(ip);
}

/// 进程内 HashMap 兜底实现（Redis 不可用时的降级路径）。
fn record_login_failure_local(ip: SocketAddr) {
    let now = chrono::Utc::now().timestamp();
    let mut map = LOGIN_FAILURES.lock().unwrap();
    sweep_stale_login_failures(&mut map, now);
    // 同 check_login_rate_limit_local：key 仅取 IP 不含临时端口
    let entry = map.entry(ip.ip().to_string()).or_insert((0, now));
    if now.saturating_sub(entry.1) >= LOGIN_RATE_LIMIT_WINDOW_SECS as i64 {
        *entry = (0, now);
    }
    entry.0 += 1;
}

/// 写登录审计日志（成功/失败统一入口）。失败路径（限流命中、用户不存在、
/// QQ 未注册、密码错误）同样落库保证审计完整；user_id 未知时记 None。
/// 审计写入失败不影响登录主流程（调用方按需忽略/透传）。
async fn record_login_log(
    user_id: Option<i64>,
    ip: SocketAddr,
    user_agent: &str,
    is_error: bool,
) -> Result<()> {
    models::system::sys_action_log::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置；操作者即登录主体
        //（未知用户名的失败登录记 0）
        create_time: Set(chrono::Utc::now().naive_utc()),
        update_time: Set(Some(chrono::Utc::now().naive_utc())),
        creator_id: Set(Some(user_id.unwrap_or(0))),
        updater_id: Set(Some(user_id.unwrap_or(0))),
        del_flag: Set(false),
        user_id: Set(user_id),
        ipv4: Set(Some(ip.ip().to_string())),
        device_id: Set(user_agent.to_string()),
        action: Set(SystemActionLogAction::Login),
        is_error: Set(is_error),
        extra_data: Set(Some(Default::default())),
    }
    .insert(&DB_CONN.wait().pg_conn)
    .await
    .map_err(internal_error)?;
    Ok(())
}

pub async fn oauth_password_login(
    username: String,
    password_raw: String,
    ip: SocketAddr,
    user_agent: String,
) -> Result<OauthLoginResponse> {
    // 限流检查在用户名查询之前：避免攻击者用无效用户名做无代价探测。
    if let Err(e) = check_login_rate_limit(ip).await {
        // 限流命中同样写失败审计日志（user_id 未知，记 None）
        let _ = record_login_log(None, ip, &user_agent, true).await;
        return Err(e);
    }

    let item = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .filter(models::system::sys_user::Column::Username.eq(username))
        .one(&DB_CONN.wait().pg_conn)
        .await
        .map_err(internal_error)?;
    let Some(item) = item else {
        // 与“密码错误”返回同一文案并同样计入失败限流，
        // 避免用户名枚举与无代价探测。
        record_login_failure(ip).await;
        // 用户不存在同样写失败审计日志（user_id 未知，记 None）
        let _ = record_login_log(None, ip, &user_agent, true).await;
        return Err(anyhow!("Invalid username or password"));
    };
    let user_id = item.id;

    let ret = oauth_password_login_inner(item, password_raw, ip, &user_agent).await;

    if ret.is_err() {
        record_login_failure(ip).await;
    } else {
        // 登录成功：登记设备（幂等 upsert）
        let _ = record_device(user_id, ip, &user_agent).await;
    }

    // 成功/失败（密码错误）统一写审计日志
    record_login_log(Some(user_id), ip, &user_agent, ret.is_err()).await?;

    ret
}

/// 把请求中的 scope 字符串映射为 `OauthScopeType`。
/// 当前仅支持 `all`（默认，大小写不敏感）；未知 scope 直接拒绝，避免
/// 静默降级为全局权限。
pub fn map_scope(scope: &str) -> Result<OauthScopeType> {
    let normalized = scope.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" | "all" => Ok(OauthScopeType::All),
        other => Err(anyhow!("Unsupported scope: {other}")),
    }
}

pub async fn oauth_client_credentials(scope: String) -> Result<OauthAnonymousResponse> {
    // 使用系统匿名用户 id = 0 表示客户端凭据/匿名访问
    let id: i64 = 0;

    let scope_enum = map_scope(&scope)?;

    // access/refresh 用不同 jti 并带 token_type 声明（与 issue_token 一致）
    let access_jti = Uuid::now_v7();
    let refresh_jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token =
        generate_token(now, id, access_jti, "access", chrono::Duration::days(15)).await?;
    let _refresh_token =
        generate_token(now, id, refresh_jti, "refresh", chrono::Duration::days(30)).await?;

    // Redis 不可用时静默跳过（与 issue_token 一致）：降级模式下
    // `oauth_parse_token` 对 sub=0 特判构造匿名 VO，不发 Redis 键也能解析；
    // REDIS_REQUIRED=true 的 fail-closed 语义由解析侧（oauth_parse_token）保证。
    if let Some(redis_client) = &DB_CONN.wait().redis_conn
        && let Ok(mut redis_conn) = redis_client.get_multiplexed_async_connection().await
    {
        // 对于匿名/客户端凭据，存储一个空的 payload 或简单标记
        let payload = serde_json::to_string(&serde_json::json!({"anon": true}))?;

        let _ = redis_conn
            .set_options(
                format!("jwt:access:{}:{}", id, access_jti),
                payload,
                SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::EX(
                        EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                    )),
            )
            .await;
        let _ = redis_conn
            .set_options(
                format!("jwt:refresh:{}:{}", id, refresh_jti),
                access_jti.to_string(),
                SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::EX(
                        REFRESH_APPEND_DURATION.as_seconds_f32() as u64,
                    )),
            )
            .await;
    }

    Ok(OauthAnonymousResponse {
        access_token,
        token_type: OauthTokenType::Bearer,
        expires_in: EXPIRED_APPEND_DURATION.as_seconds_f32() as i64,
        scope: scope_enum,
        jti: access_jti,
    })
}

/// 生成 JWKS（JSON Web Key Set），向第三方分发当前 JWT 密钥的公钥信息。
///
/// 签名算法由 `JWT_RSA_PRIVATE_KEY_PEM` 决定：配置了 RSA 私钥时签发 RS256
/// 并以 `kty: RSA` 公布公钥；否则维持 HS256，以 `kty: oct` 公布 HMAC 密钥
/// （RFC 7517 §6.4）。
pub async fn do_jwks() -> Result<serde_json::Value> {
    _utils::jwt::jwks()
}

pub async fn oauth_refresh(
    refresh_token: String,
    ip: SocketAddr,
    user_agent: String,
) -> Result<OauthLoginResponse> {
    // 验证传入的 refresh token 并获取 claims
    let claims = verify_token(&refresh_token).await?;

    // S2：只接受 refresh 类型令牌。access token 直接拒绝；旧格式令牌
    // （无 token_type 声明）一律拒绝，强制重新登录（旧格式即本漏洞来源）。
    if claims.token_type.as_deref() != Some("refresh") {
        return Err(anyhow!("Not a refresh token"));
    }

    // 匿名身份（sub=0）从不签发 refresh token（OauthAnonymousResponse 无
    // refresh_token 字段），提前拒绝，避免查询不存在的用户行。
    if claims.sub == 0 {
        return Err(anyhow!("Anonymous identity cannot be refreshed"));
    }

    let user = models::system::sys_user::Entity::find_safety_by_id(claims.sub)
        .one(&DB_CONN.wait().pg_conn)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;
    let vo: SysUserVO = user.clone().into();

    // 与登录一致地校验 access_policy（IP / 设备绑定策略）：对齐 Java
    // `checkDeviceAccess`（token enhancer 对所有签发路径生效）—— 策略违规
    // 只影响审计日志与提示信息，登录/刷新仍然成功（「警告放行」）。
    // 无策略用户（空 policy）天然放行；SKIP_ACCESS_POLICY=true 时跳过。
    let policy: Vec<_> = user.access_policy.clone().map(|a| a.0).unwrap_or_default();
    if let Err(e) = check_access_policy(user.id, &policy, ip, &user_agent).await {
        tracing::warn!(user_id = user.id, error = %e, "access policy violation (allowed through, Java-compatible)");
    }

    // 降级策略（与 oauth_parse_token 一致，补齐与密码登录 issue_token 的
    // 对齐，M1）：
    // - Redis 可达：原子 GETDEL 轮换（旧 refresh 立即作废，吊销旧 access）；
    // - REDIS_REQUIRED=true 且 Redis 不可达：fail-closed，拒绝刷新；
    // - REDIS_REQUIRED=false 且 Redis 不可达：降级为「只验 JWT 签名 +
    //   token_type + 用户存在」，**不轮换**（旧 refresh token 仍有效），
    //   直接签发新 token 对（issue_token 内部同样静默跳过 Redis 存储）。
    let redis_required = std::env::var("REDIS_REQUIRED").as_deref() == Ok("true");
    let redis_conn = match &DB_CONN.wait().redis_conn {
        Some(client) => match client.get_multiplexed_async_connection().await {
            Ok(conn) => Some(conn),
            Err(_) if redis_required => {
                return Err(anyhow!("Redis unavailable (REDIS_REQUIRED=true)"));
            },
            Err(_) => None,
        },
        None if redis_required => {
            return Err(anyhow!("Redis unavailable (REDIS_REQUIRED=true)"));
        },
        None => None,
    };
    let Some(mut redis_conn) = redis_conn else {
        return issue_token(&user).await;
    };

    // 原子 claim 旧 refresh key（GETDEL）：返回 None 说明 key 已不存在
    // （已被轮换/吊销/登出），拒绝重放；同时取出配对 access_jti。
    let refresh_key = format!("jwt:refresh:{}:{}", claims.sub, claims.jti);
    let old_access_jti: Option<String> = redis_conn
        .get_del(&refresh_key)
        .await
        .map_err(internal_error)?;
    if old_access_jti.is_none() {
        return Err(anyhow!("Refresh token not found or already used"));
    }

    // 吊销旧 access token（其 jti 记录在 refresh key 的 value 中）
    if let Some(old_access_jti) = old_access_jti.as_deref()
        && !old_access_jti.is_empty()
    {
        let _deleted: usize = redis_conn
            .del(format!("jwt:access:{}:{}", claims.sub, old_access_jti))
            .await
            .map_err(internal_error)?;
    }

    // 生成新的 jti 和 token（旧 refresh 已作废：轮换后立即失效）
    let new_access_jti = Uuid::now_v7();
    let new_refresh_jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(
        now,
        claims.sub,
        new_access_jti,
        "access",
        chrono::Duration::days(15),
    )
    .await?;
    let refresh_token_new = generate_token(
        now,
        claims.sub,
        new_refresh_jti,
        "refresh",
        chrono::Duration::days(30),
    )
    .await?;

    // 写入新的 access/refresh 条目（payload 取 DB 最新用户信息，
    // 不再依赖旧 access key 的缓存内容）
    let access_key = format!("jwt:access:{}:{}", claims.sub, new_access_jti);
    let refresh_key_new = format!("jwt:refresh:{}:{}", claims.sub, new_refresh_jti);

    redis_conn
        .set_options(
            &access_key,
            serde_json::to_string(&vo)?,
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await
        .map_err(internal_error)?;

    redis_conn
        .set_options(
            &refresh_key_new,
            new_access_jti.to_string(),
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    REFRESH_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await
        .map_err(internal_error)?;

    Ok(OauthLoginResponse {
        access_token,
        refresh_token: refresh_token_new,
        token_type: OauthTokenType::Bearer,
        expires_in: EXPIRED_APPEND_DURATION.as_seconds_f32() as i64,
        scope: OauthScopeType::All,
        jti: new_access_jti,
        user_id: claims.sub,
        user_roles: vec![role_code(user.role_id)],
        env: Some(environment_id()),
        message: Some(String::new()),
    })
}
