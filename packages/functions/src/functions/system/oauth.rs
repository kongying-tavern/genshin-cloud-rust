use anyhow::{Result, anyhow};
use std::net::SocketAddr;

use redis::{AsyncTypedCommands, SetOptions};
use sea_orm::{ActiveValue::Set, QueryOrder, prelude::*};

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
            id: Set(0),
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
    check_access_policy(item.id, &item.access_policy.0, ip, user_agent).await?;

    let jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(now, item.id, jti).await?;
    let refresh_token = generate_token(now, item.id, jti).await?;

    let id = item.id;
    let vo: SysUserVO = item.into();

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

pub async fn oauth_password_login(
    username: String,
    password_raw: String,
    ip: SocketAddr,
    user_agent: String,
) -> Result<OauthLoginResponse> {
    let item = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .filter(models::system::sys_user::Column::Username.eq(username))
        .one(&DB_CONN.wait().pg_conn)
        .await?
        .ok_or(anyhow!("User not found"))?;
    let user_id = item.id;

    let ret = oauth_password_login_inner(item, password_raw, ip, &user_agent).await;

    if ret.is_ok() {
        // 登录成功：登记设备（幂等 upsert）
        let _ = record_device(user_id, ip, &user_agent).await;
    }

    models::system::sys_action_log::ActiveModel {
        version: Set(0),
        id: Set(0),
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
        extra_data: Set(Default::default()),
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
/// 当前使用 HMAC-SHA256（HS256）签名，JWKS 以 `kty: oct` 形式公布 HMAC 密钥
/// （RFC 7517 §6.4）。若未来切换到 RS256（如 JWK 轮换），此端点改为分发
/// RSA 公钥。
pub async fn do_jwks() -> Result<serde_json::Value> {
    use base64::Engine;

    let key = _utils::jwt::jwt_secret_raw();
    let k = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(key.as_bytes());
    Ok(serde_json::json!({
        "keys": [{
            "kty": "oct",
            "kid": "genshin-cloud-hmac-v1",
            "alg": "HS256",
            "use": "sig",
            "k": k,
        }],
    }))
}

pub async fn oauth_refresh(refresh_token: String) -> Result<()> {
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

    // 生成新的 jti 和 token
    let new_jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let _new_access = generate_token(now, claims.sub, new_jti).await?;
    let _new_refresh = generate_token(now, claims.sub, new_jti).await?;

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

    Ok(())
}
