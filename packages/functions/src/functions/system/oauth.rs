use anyhow::{anyhow, Result};
use std::net::SocketAddr;

use redis::{AsyncTypedCommands, SetOptions};
use sea_orm::{prelude::*, ActiveValue::Set};

use _database::{models, DB_CONN};
use _utils::{
    bcrypt::verify_hash,
    jwt::{generate_token, verify_token, Claims, EXPIRED_APPEND_DURATION},
    models::SysUserVO,
    types::{
        auth::{OauthAnonymousResponse, OauthLoginResponse, OauthScopeType, OauthTokenType},
        SystemActionLogAction,
    },
};

async fn oauth_password_login_inner(
    item: models::system::sys_user::Model,
    password_raw: String,
) -> Result<OauthLoginResponse> {
    if !verify_hash(
        password_raw,
        item.password
            .strip_prefix("{bcrypt}")
            .ok_or(anyhow!("Failed to strip bcrypt prefix"))?,
    )? {
        return Err(anyhow!("Invalid password"));
    }

    let jti = Uuid::now_v7();
    let now = chrono::Utc::now();
    let access_token = generate_token(now, item.id, jti).await?;
    let refresh_token = generate_token(now, item.id, jti).await?;

    let id = item.id;
    let vo: SysUserVO = item.into();
    let mut redis_conn = DB_CONN
        .wait()
        .redis_conn
        .get_multiplexed_async_connection()
        .await?;
    redis_conn
        .set_options(
            format!("jwt:access:{}:{}", id, jti),
            serde_json::to_string(&vo)?,
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
            &"",
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await?;

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

    let mut redis_conn = DB_CONN
        .wait()
        .redis_conn
        .get_multiplexed_async_connection()
        .await?;
    let item = redis_conn
        .get(format!("jwt:access:{}:{}", claims.sub, claims.jti))
        .await?
        .ok_or(anyhow!("Token not found in Redis"))?;
    Ok((serde_json::from_str(&item)?, claims))
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

    // TODO: 根据 access_policy 指定的模式，对请求做各种检查
    let ret = oauth_password_login_inner(item, password_raw).await;

    models::system::sys_action_log::ActiveModel {
        user_id: Set(Some(user_id)),
        ipv4: Set(Some(ip.to_string())),
        device_id: Set(user_agent),
        action: Set(SystemActionLogAction::Login),
        is_error: Set(ret.is_err()),
        extra_data: Set(Default::default()),
        ..Default::default()
    }
    .insert(&DB_CONN.wait().pg_conn)
    .await?;

    ret
}

pub async fn oauth_client_credentials(scope: String) -> Result<OauthAnonymousResponse> {
    // Handle client credentials grant type
    todo!()
}

pub async fn oauth_refresh(refresh_token: String) -> Result<()> {
    todo!()
}
