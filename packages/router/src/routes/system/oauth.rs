use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, Json, Multipart, Query, State},
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

/// OAuth2 错误响应：JSON 体 `{"error": "...", "error_description": "..."}`
/// （Spring OAuth2 契约；前端 axios 按 `error_description` 提取文案）。
pub type OAuthError = (StatusCode, axum::Json<serde_json::Value>);

fn oauth_error(code: StatusCode, error: &str, description: &str) -> OAuthError {
    (
        code,
        axum::Json(serde_json::json!({
            "error": error,
            "error_description": description,
        })),
    )
}

#[tracing::instrument(skip(db, req))]
pub async fn oauth(
    State(db): State<crate::routes::SharedDb>,
    ConnectInfo(native_ip): ConnectInfo<SocketAddr>,
    ExtractIP(ip): ExtractIP,
    ExtractUserAgent(user_agent): ExtractUserAgent,
    Query(query): Query<LoginQuery>,
    req: axum::extract::Request,
) -> Result<impl IntoResponse, OAuthError> {
    use axum::extract::FromRequest;
    use axum::http::header::CONTENT_TYPE;

    let ip = ip.unwrap_or(native_ip);

    // 表单字段的三种到达通道（Java Spring OAuth2 全兼容：GET/POST ×
    // query/form，form 同时接受 multipart 与 x-www-form-urlencoded）：
    // - multipart/form-data（前端 FormData 登录）；
    // - application/x-www-form-urlencoded（标准 OAuth2 客户端与旧前端的
    //   refresh 请求）；
    // - 其余（JSON / 空 body）：字段只可能在 query 里。
    let (parts, body) = req.into_parts();
    let content_type = parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let bytes = axum::body::to_bytes(body, 1024 * 1024)
        .await
        .map_err(|err| {
            oauth_error(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                &format!("Failed to read body: {err}"),
            )
        })?;
    let req = axum::extract::Request::from_parts(parts, axum::body::Body::from(bytes));

    let form_fields: HashMap<String, String> = if content_type.starts_with("multipart/form-data") {
        let mut ret = HashMap::new();
        if let Ok(mut form) = Multipart::from_request(req, &()).await {
            while let Some(field) = form.next_field().await.map_err(|err| {
                oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    &format!("Failed to read form field: {err}"),
                )
            })? {
                let name = field
                    .name()
                    .ok_or_else(|| {
                        oauth_error(
                            StatusCode::BAD_REQUEST,
                            "invalid_request",
                            "Field name is required",
                        )
                    })?
                    .to_string();
                let value = field.text().await.map_err(|err| {
                    oauth_error(
                        StatusCode::BAD_REQUEST,
                        "invalid_request",
                        &format!("Failed to read form field {name}: {err}"),
                    )
                })?;
                ret.insert(name, value);
            }
        }
        ret
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        axum::extract::Form::from_request(req, &())
            .await
            .map(|axum::extract::Form(map)| map)
            .unwrap_or_default()
    } else {
        HashMap::new()
    };
    // 统一参数视图：form 字段优先、query 兜底 —— Java Spring OAuth2 的
    // GET/POST × query/form 四种组合全兼容。password 的 username/password
    // 只可能来自 form（query 里没有这两个字段，避免把口令塞进 URL）。
    let param = |name: &str| -> Option<String> {
        form_fields.get(name).cloned().or_else(|| match name {
            "grant_type" => query.grant_type.clone(),
            "scope" => query.scope.clone(),
            "refresh_token" => query.refresh_token.clone(),
            _ => None,
        })
    };
    let grant_type = param("grant_type")
        .map(|g| g.trim().to_string())
        .ok_or_else(|| {
            oauth_error(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "Grant type is required",
            )
        })?;
    match grant_type.as_str() {
        "password" => {
            let username = param("username").ok_or_else(|| {
                oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "Username is required",
                )
            })?;
            let password = param("password").ok_or_else(|| {
                oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "Password is required",
                )
            })?;
            return Ok(Json(
                oauth_password_login(db.as_ref(), username, password, ip, user_agent)
                    .await
                    .map_err(|e| {
                        // Java/Spring OAuth2 契约：账密失败 -> 400 invalid_grant
                        // "Bad credentials"（前端据此映射「账号或密码错误」）。
                        tracing::warn!("password grant failed: {e}");
                        oauth_error(StatusCode::BAD_REQUEST, "invalid_grant", "Bad credentials")
                    })?,
            )
            .into_response());
        },
        "client_credentials" => {
            let scope = param("scope").ok_or_else(|| {
                oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "Scope is required for client credentials",
                )
            })?;
            return Ok(Json(
                oauth_client_credentials(db.as_ref(), ip, scope)
                    .await
                    .map_err(|e| {
                        tracing::warn!("client_credentials grant failed: {e}");
                        oauth_error(StatusCode::BAD_REQUEST, "invalid_scope", &e.to_string())
                    })?,
            )
            .into_response());
        },
        "refresh_token" => {
            let refresh_token = param("refresh_token").ok_or_else(|| {
                oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "Refresh token is required for refresh token grant type",
                )
            })?;
            let ret = oauth_refresh(db.as_ref(), refresh_token, ip, user_agent)
                .await
                .map_err(|e| {
                    tracing::warn!("refresh grant failed: {e}");
                    oauth_error(StatusCode::BAD_REQUEST, "invalid_grant", &e.to_string())
                })?;
            return Ok(Json(ret).into_response());
        },
        _ => {
            return Err(oauth_error(
                StatusCode::BAD_REQUEST,
                "unsupported_grant_type",
                "Invalid grant type",
            ));
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
