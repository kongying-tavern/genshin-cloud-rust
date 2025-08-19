use anyhow::Result;
use serde::Deserialize;
use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Form, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::{ExtractIP, ExtractUserAgent};
use _functions::functions::system::oauth::{
    oauth_client_credentials, oauth_password_login, oauth_refresh,
};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum LoginParams {
    Password { username: String, password: String },
    ClientCredentials { scope: String },
    RefreshToken { refresh_token: String },
}

#[tracing::instrument(skip_all)]
pub async fn oauth(
    ConnectInfo(native_ip): ConnectInfo<SocketAddr>,
    ExtractIP(ip): ExtractIP,
    ExtractUserAgent(user_agent): ExtractUserAgent,
    Query(query): Query<Option<LoginParams>>,
    Form(form): Form<Option<LoginParams>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ip = ip.unwrap_or(native_ip);

    if let Some(form) = form {
        match form {
            LoginParams::Password { username, password } => Ok(Json(
                oauth_password_login(username, password)
                    .await
                    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?,
            )
            .into_response()),
            _ => return Err((StatusCode::BAD_REQUEST, "Invalid grant type".into())),
        }
    } else if let Some(query) = query {
        match query {
            LoginParams::ClientCredentials { scope } => Ok(Json(
                oauth_client_credentials(scope)
                    .await
                    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?,
            )
            .into_response()),
            LoginParams::RefreshToken { refresh_token } => {
                oauth_refresh(refresh_token)
                    .await
                    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
                Ok(().into_response())
            }
            _ => return Err((StatusCode::BAD_REQUEST, "Invalid grant type".into())),
        }
    } else {
        return Err((StatusCode::BAD_REQUEST, "No parameters provided".into()));
    }
}
