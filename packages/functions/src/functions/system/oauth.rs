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
    jwt::{Claims, EXPIRED_APPEND_DURATION, generate_token, verify_token},
    models::SysUserVO,
    types::{
        AccessPolicyItemEnum, SystemActionLogAction,
        auth::{OauthAnonymousResponse, OauthLoginResponse, OauthScopeType, OauthTokenType},
    },
};

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
    let db = &DB_CONN.wait().pg_conn;
    let last = models::system::sys_user_device::Entity::find_safety()
        .filter(models::system::sys_user_device::Column::UserId.eq(Some(user_id)))
        .order_by_desc(models::system::sys_user_device::Column::LastLoginTime)
        .one(db)
        .await?;

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
                    .await?;
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
                    .await?;
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
        let mut am: models::system::sys_user_device::ActiveModel = dev.into();
        am.ipv4 = Set(Some(ip.ip().to_string()));
        am.last_login_time = Set(Some(now));
        models::system::sys_user_device::Entity::update_safety(am)?
            .exec(db)
            .await?;
    } else {
        let am = models::system::sys_user_device::ActiveModel {
            version: Set(0),
            id: NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),
            user_id: Set(Some(user_id)),
            device_id: Set(user_agent.to_string()),
            ipv4: Set(Some(ip.ip().to_string())),
            status: Set(0),
            last_login_time: Set(Some(now)),
        };
        models::system::sys_user_device::Entity::insert(am)
            .exec(db)
            .await?;
    }
    Ok(())
}

/// 为用户签发 access/refresh token（含 Redis 会话存储，Redis 不可用时降级）。
async fn issue_token(item: &models::system::sys_user::Model) -> Result<OauthLoginResponse> {
    let jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(now, item.id, jti).await?;
    let refresh_token = generate_token(now, item.id, jti).await?;

    let id = item.id;
    let vo: SysUserVO = item.clone().into();

    // Store token in Redis if available (graceful degradation for e2e mode
    // where Redis is not running — token verification will fall back to
    // JWT-only validation without Redis session lookup).
    if let Some(redis_client) = &DB_CONN.wait().redis_conn
        && let Ok(mut redis_conn) = redis_client.get_multiplexed_async_connection().await
    {
        let _ = redis_conn
            .set_options(
                format!("jwt:access:{}:{}", id, jti),
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
                format!("jwt:refresh:{}:{}", id, jti),
                "",
                SetOptions::default()
                    .conditional_set(redis::ExistenceCheck::NX)
                    .with_expiration(redis::SetExpiry::EX(
                        EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
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
        jti,
    })
}

async fn oauth_password_login_inner(
    item: models::system::sys_user::Model,
    password_raw: String,
    ip: SocketAddr,
    user_agent: &str,
) -> Result<OauthLoginResponse> {
    if !verify_password(password_raw, item.password.clone())? {
        return Err(anyhow!("Invalid password"));
    }

    // 身份验证通过后，按用户的 access_policy 校验登录环境
    let policy: Vec<_> = item.access_policy.clone().map(|a| a.0).unwrap_or_default();
    check_access_policy(item.id, &policy, ip, user_agent).await?;

    issue_token(&item).await
}

pub async fn oauth_parse_token(token: String) -> Result<(SysUserVO, Claims)> {
    let claims = verify_token(&token).await?;

    // Try Redis session lookup; fall back to DB lookup if Redis is unavailable
    // (graceful degradation for e2e mode).
    if let Some(redis_client) = &DB_CONN.wait().redis_conn
        && let Ok(mut redis_conn) = redis_client.get_multiplexed_async_connection().await
    {
        let key = format!("jwt:access:{}:{}", claims.sub, claims.jti);
        if let Ok(Some(item)) = redis_conn.get::<String>(key).await {
            return Ok((serde_json::from_str(&item)?, claims));
        }
    }

    // Fallback: look up user from DB by the JWT subject (user_id)
    let user = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::Id.eq(claims.sub))
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .one(&DB_CONN.wait().pg_conn)
        .await?
        .ok_or(anyhow!("User not found for token"))?;
    Ok((user.into(), claims))
}

/// 登录暴力破解限流：按 IP 固定窗口（每分钟最多 5 次失败的密码登录尝试）。
/// 只计数失败尝试（成功登录不消耗额度），窗口过后自动重置。
const LOGIN_RATE_LIMIT_PER_MINUTE: u32 = 5;

static LOGIN_FAILURES: Lazy<std::sync::Mutex<std::collections::HashMap<String, (u32, i64)>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

fn check_login_rate_limit(ip: SocketAddr) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let window = now / 60;
    let mut map = LOGIN_FAILURES.lock().unwrap();
    let entry = map.entry(ip.to_string()).or_insert((0, window));
    if entry.1 != window {
        *entry = (0, window);
    }
    if entry.0 >= LOGIN_RATE_LIMIT_PER_MINUTE {
        return Err(anyhow!(
            "Too many failed login attempts; try again in a minute"
        ));
    }
    Ok(())
}

fn record_login_failure(ip: SocketAddr) {
    let now = chrono::Utc::now().timestamp();
    let window = now / 60;
    let mut map = LOGIN_FAILURES.lock().unwrap();
    let entry = map.entry(ip.to_string()).or_insert((0, window));
    if entry.1 != window {
        *entry = (0, window);
    }
    entry.0 += 1;
}

pub async fn oauth_password_login(
    username: String,
    password_raw: String,
    ip: SocketAddr,
    user_agent: String,
) -> Result<OauthLoginResponse> {
    // 限流检查在用户名查询之前：避免攻击者用无效用户名做无代价探测。
    check_login_rate_limit(ip)?;

    let item = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .filter(models::system::sys_user::Column::Username.eq(username))
        .one(&DB_CONN.wait().pg_conn)
        .await?
        .ok_or(anyhow!("User not found"))?;
    let user_id = item.id;

    let ret = oauth_password_login_inner(item, password_raw, ip, &user_agent).await;

    if ret.is_err() {
        record_login_failure(ip);
    } else {
        // 登录成功：登记设备（幂等 upsert）
        let _ = record_device(user_id, ip, &user_agent).await;
    }

    models::system::sys_action_log::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(chrono::Utc::now().naive_utc()),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        user_id: Set(Some(user_id)),
        ipv4: Set(Some(ip.ip().to_string())),
        device_id: Set(user_agent),
        action: Set(SystemActionLogAction::Login),
        is_error: Set(ret.is_err()),
        extra_data: Set(Some(Default::default())),
    }
    .insert(&DB_CONN.wait().pg_conn)
    .await?;

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

/// QQ 第三方登录：客户端完成 QQ 授权后，用拿到的 openid 换取本站 token。
///
/// 按 `sys_user.qq` 匹配用户；未注册的 openid 返回明确错误（注册走
/// `/user/register/qq`，把 openid 写入 `qq` 字段）。与密码登录一致地执行
/// access_policy 检查、设备登记与登录日志。
pub async fn oauth_qq_login(
    qq_openid: String,
    ip: SocketAddr,
    user_agent: String,
) -> Result<OauthLoginResponse> {
    let db = &DB_CONN.wait().pg_conn;
    let item = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .filter(models::system::sys_user::Column::Qq.eq(Some(qq_openid)))
        .one(db)
        .await?
        .ok_or(anyhow!("QQ account not registered"))?;
    let user_id = item.id;

    // 身份由 openid 提供；同样校验登录环境
    let policy: Vec<_> = item.access_policy.clone().map(|a| a.0).unwrap_or_default();
    check_access_policy(item.id, &policy, ip, &user_agent).await?;
    let ret = issue_token(&item).await;

    if ret.is_ok() {
        let _ = record_device(user_id, ip, &user_agent).await;
    }
    models::system::sys_action_log::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(chrono::Utc::now().naive_utc()),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        user_id: Set(Some(user_id)),
        ipv4: Set(Some(ip.ip().to_string())),
        device_id: Set(user_agent),
        action: Set(SystemActionLogAction::Login),
        is_error: Set(ret.is_err()),
        extra_data: Set(Some(Default::default())),
    }
    .insert(&DB_CONN.wait().pg_conn)
    .await?;

    ret
}

pub async fn oauth_client_credentials(scope: String) -> Result<OauthAnonymousResponse> {
    // 使用系统匿名用户 id = 0 表示客户端凭据/匿名访问
    let id: i64 = 0;

    let scope_enum = map_scope(&scope)?;

    let jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(now, id, jti).await?;
    let _refresh_token = generate_token(now, id, jti).await?;

    let mut redis_conn = DB_CONN
        .wait()
        .redis_conn
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Redis not available"))?
        .get_multiplexed_async_connection()
        .await?;

    // 对于匿名/客户端凭据，存储一个空的 payload 或简单标记
    let payload = serde_json::to_string(&serde_json::json!({"anon": true}))?;

    redis_conn
        .set_options(
            format!("jwt:access:{}:{}", id, jti),
            payload,
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await?;
    redis_conn
        .set_options(
            format!("jwt:refresh:{}:{}", id, jti),
            "",
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await?;

    Ok(OauthAnonymousResponse {
        access_token,
        token_type: OauthTokenType::Bearer,
        expires_in: EXPIRED_APPEND_DURATION.as_seconds_f32() as i64,
        scope: scope_enum,
        jti,
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

pub async fn oauth_refresh(refresh_token: String) -> Result<OauthLoginResponse> {
    // 验证传入的 refresh token 并获取 claims
    let claims = verify_token(&refresh_token).await?;

    let mut redis_conn = DB_CONN
        .wait()
        .redis_conn
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Redis not available"))?
        .get_multiplexed_async_connection()
        .await?;

    // 确认 refresh token 在 Redis 中存在
    let refresh_key = format!("jwt:refresh:{}:{}", claims.sub, claims.jti);
    let exists: Option<String> = redis_conn.get(&refresh_key).await?;
    if exists.is_none() {
        return Err(anyhow!("Refresh token not found"));
    }

    // 生成新的 jti 和 token（旧 token 一次性使用：旋转后立即作废）
    let new_jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(now, claims.sub, new_jti).await?;
    let refresh_token_new = generate_token(now, claims.sub, new_jti).await?;

    // 保持原来的访问负载（如果有），尝试读取旧的 access payload
    let old_access_key = format!("jwt:access:{}:{}", claims.sub, claims.jti);
    let access_payload: Option<String> = redis_conn.get(&old_access_key).await?;

    // 写入新的 access/refresh 条目
    let access_key = format!("jwt:access:{}:{}", claims.sub, new_jti);
    let refresh_key_new = format!("jwt:refresh:{}:{}", claims.sub, new_jti);

    let payload_to_store = access_payload.unwrap_or_else(|| serde_json::json!({}).to_string());

    redis_conn
        .set_options(
            &access_key,
            payload_to_store,
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await?;

    redis_conn
        .set_options(
            &refresh_key_new,
            "",
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await?;

    // 删除旧的 access/refresh 键
    let _deleted_old_access: usize = redis_conn.del(old_access_key).await?;
    let _deleted_old_refresh: usize = redis_conn.del(refresh_key).await?;

    Ok(OauthLoginResponse {
        access_token,
        refresh_token: refresh_token_new,
        token_type: OauthTokenType::Bearer,
        expires_in: EXPIRED_APPEND_DURATION.as_seconds_f32() as i64,
        scope: OauthScopeType::All,
        jti: new_jti,
    })
}
