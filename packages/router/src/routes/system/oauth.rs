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
    oauth_client_credentials, oauth_password_login, oauth_refresh,
};

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    grant_type: Option<String>,
    // ClientCredentials
    scope: Option<String>,
    // RefreshToken
    refresh_token: Option<String>,
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

    // grant_type 手动按字符串分派（password 走上面的 multipart 表单；client_credentials
    // / refresh_token 走 query）：serde_urlencoded 对 query 里的单元枚举
    // （untagged enum）无法反序列化——此前用类型化枚举导致所有 query 型
    // grant_type 一律在 Query 提取器阶段 400，refresh_token / client_credentials
    // 两个 HTTP 分支完全不可用。
    match query.grant_type.as_deref().map(str::trim) {
        Some("client_credentials") => {
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
        Some("refresh_token") => {
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
        _ => {
            return Err((StatusCode::BAD_REQUEST, "Invalid grant type".into()));
        },
    }
}

#[cfg(test)]
mod tests {
    use super::LoginQuery;
    use axum::extract::Query;

    /// The frontend refresh grant posts `/oauth/token?grant_type=refresh_token&…`
    /// with a JSON content type (no form body); client credentials similarly
    /// arrive as query params. The `LoginQuery` shape must deserialize the
    /// exact query strings the frontend sends — an earlier typed enum variant
    /// rejected every value at the extractor stage and silently killed token
    /// refresh for all clients.
    #[test]
    fn login_query_parses_frontend_refresh_and_client_credentials_params() {
        let uri = "/oauth/token?grant_type=refresh_token&refresh_token=abc.def"
            .parse::<axum::http::Uri>()
            .unwrap();
        let Query(q) = Query::<LoginQuery>::try_from_uri(&uri).expect("refresh query must parse");
        assert_eq!(q.grant_type.as_deref(), Some("refresh_token"));
        assert_eq!(q.refresh_token.as_deref(), Some("abc.def"));

        let uri = "/oauth/token?grant_type=client_credentials&scope=all"
            .parse::<axum::http::Uri>()
            .unwrap();
        let Query(q) =
            Query::<LoginQuery>::try_from_uri(&uri).expect("client_credentials query must parse");
        assert_eq!(q.grant_type.as_deref(), Some("client_credentials"));
        assert_eq!(q.scope.as_deref(), Some("all"));

        let uri = "/oauth/token".parse::<axum::http::Uri>().unwrap();
        let Query(q) = Query::<LoginQuery>::try_from_uri(&uri).expect("bare query must parse");
        assert!(q.grant_type.is_none());
    }
}
