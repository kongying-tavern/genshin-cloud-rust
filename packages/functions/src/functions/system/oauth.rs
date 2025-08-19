use anyhow::{anyhow, Result};

use redis::{AsyncTypedCommands, SetOptions};
use sea_orm::prelude::*;

use _database::{models, DB_CONN};
use _utils::{
    bcrypt::verify_hash,
    jwt::{generate_token, EXPIRED_APPEND_DURATION},
    types::auth::{OauthAnonymousResponse, OauthLoginResponse, OauthScopeType, OauthTokenType},
};

pub async fn oauth_password_login(
    username: String,
    password: String,
) -> Result<OauthLoginResponse> {
    let item = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::DelFlag.eq(false))
        .filter(models::system::sys_user::Column::Username.eq(username))
        .one(&DB_CONN.wait().pg_conn)
        .await?
        .ok_or(anyhow!("User not found"))?;

    if !verify_hash(
        password,
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

    let mut redis_conn = DB_CONN
        .wait()
        .redis_conn
        .get_multiplexed_async_connection()
        .await?;
    redis_conn
        .set_options(
            format!("jwt:access:{}:{}", item.id, jti),
            &access_token,
            SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(
                    EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
                )),
        )
        .await?;
    redis_conn
        .set_options(
            format!("jwt:refresh:{}:{}", item.id, jti),
            &refresh_token,
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
        expires_in: EXPIRED_APPEND_DURATION.as_seconds_f32() as u64,
        scope: OauthScopeType::All,
        jti,
    })
}

pub async fn oauth_client_credentials(scope: String) -> Result<OauthAnonymousResponse> {
    // Handle client credentials grant type
    todo!()
}

pub async fn oauth_refresh(refresh_token: String) -> Result<()> {
    todo!()
}
