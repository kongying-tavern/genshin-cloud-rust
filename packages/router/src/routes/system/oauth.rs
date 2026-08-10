use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, Json, Multipart, Query},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::{ExtractIP, ExtractUserAgent};
use _functions::functions::system::oauth::{
    oauth_client_credentials, oauth_password_login, oauth_qq_login, oauth_refresh,
};

#[derive(Debug, Deserialize)]
pub struct QqLoginParams {
    /// QQ 授权后客户端拿到的 openid
    pub qq_openid: String,
}

/// QQ 第三方登录
/// POST /oauth/qq
#[tracing::instrument(skip(params))]
pub async fn qq_login(
    ConnectInfo(native_ip): ConnectInfo<SocketAddr>,
    ExtractIP(ip): ExtractIP,
    ExtractUserAgent(user_agent): ExtractUserAgent,
    Json(params): Json<QqLoginParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ip = ip.unwrap_or(native_ip);
    let ret = oauth_qq_login(params.qq_openid, ip, user_agent)
        .await
        .map_err(crate::routes::internal_error)?;
    Ok(Json(ret).into_response())
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    grant_type: Option<LoginQueryType>,
    // ClientCredentials
    scope: Option<String>,
    // RefreshToken
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum LoginQueryType {
    // 这里留着只是用于标记，Password 这个项是故意忽略的，因为这个类型只能手动解析
    // Password,
    ClientCredentials,
    RefreshToken,
}

#[tracing::instrument(skip(form))]
pub async fn oauth(
    ConnectInfo(native_ip): ConnectInfo<SocketAddr>,
    ExtractIP(ip): ExtractIP,
    ExtractUserAgent(user_agent): ExtractUserAgent,
    Query(query): Query<LoginQuery>,
    form: Option<Multipart>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ip = ip.unwrap_or(native_ip);

    // 刷新 token 分支由前端以 `application/json` + query 参数请求（无 body），
    // 此时 Multipart extractor 会因 Content-Type 不匹配而 415 —— 用 Option 兜底，
    // 仅 password 模式（multipart/form-data）才消费 form 字段。
    let mut form_fields = if let Some(mut form) = form {
        let mut ret = HashMap::new();
        while let Some(field) = form.next_field().await.map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read form field: {}", err),
            )
        })? {
            let name = field
                .name()
                .ok_or((StatusCode::BAD_REQUEST, "Field name is required".into()))?
                .to_string();
            let value = field.text().await.map_err(|err| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read form field {}: {}", name, err),
                )
            })?;
            ret.insert(name, value);
        }
        ret
    } else {
        HashMap::new()
    };
    if !form_fields.is_empty() {
        let grant_type = form_fields
            .remove("grant_type")
            .ok_or((StatusCode::BAD_REQUEST, "Grant type is required".into()))?;
        if grant_type != "password" {
            return Err((StatusCode::BAD_REQUEST, "Invalid grant type".into()));
        }

        let username = form_fields
            .remove("username")
            .ok_or((StatusCode::BAD_REQUEST, "Username is required".into()))?;
        let password = form_fields
            .remove("password")
            .ok_or((StatusCode::BAD_REQUEST, "Password is required".into()))?;
        return Ok(Json(
            oauth_password_login(username, password, ip, user_agent)
                .await
                .map_err(crate::routes::internal_error)?,
        )
        .into_response());
    }

    match query
        .grant_type
        .ok_or((StatusCode::BAD_REQUEST, "Grant type is required".into()))?
    {
        LoginQueryType::ClientCredentials => {
            let scope = query.scope.ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Scope is required for client credentials".into(),
                )
            })?;
            return Ok(Json(
                oauth_client_credentials(scope)
                    .await
                    .map_err(crate::routes::internal_error)?,
            )
            .into_response());
        },
        LoginQueryType::RefreshToken => {
            let refresh_token = query.refresh_token.ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Refresh token is required for refresh token grant type".into(),
                )
            })?;
            let ret = oauth_refresh(refresh_token, ip, user_agent)
                .await
                .map_err(crate::routes::internal_error)?;
            return Ok(Json(ret).into_response());
        },
    }
}
