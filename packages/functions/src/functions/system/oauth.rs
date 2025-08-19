use anyhow::Result;

use sea_orm::prelude::*;

use _database::{models, DB_CONN};
use _utils::types::auth::{OauthAnonymousResponse, OauthLoginResponse};

pub async fn oauth_password_login(
    username: String,
    password: String,
) -> Result<OauthLoginResponse> {
    let user = models::system::sys_user::Entity::find()
        .filter(models::system::sys_user::Column::Username.eq(username))
        .one(&DB_CONN.wait().pg_conn)
        .await?;

    // Continue with the login logic

    todo!()
}

pub async fn oauth_client_credentials(scope: String) -> Result<OauthAnonymousResponse> {
    // Handle client credentials grant type
    todo!()
}

pub async fn oauth_refresh(refresh_token: String) -> Result<()> {
    todo!()
}
