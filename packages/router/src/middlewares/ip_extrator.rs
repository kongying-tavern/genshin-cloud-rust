use anyhow::Result;
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

#[derive(Debug, Clone)]
pub struct ExtractIP(pub Option<std::net::SocketAddr>);

impl<S> FromRequestParts<S> for ExtractIP
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Proxy headers (X-Real-IP / X-Forwarded-For) are only trusted when
        // the deployment explicitly enables TRUST_PROXY_HEADERS=1 (i.e. the
        // process sits behind a reverse proxy that overwrites them). By
        // default the socket address is used, so clients cannot spoof their
        // IP into access-policy checks or the action log.
        if std::env::var("TRUST_PROXY_HEADERS").is_err() {
            return Ok(Self(None));
        }

        let headers = parts.headers.clone();

        let raw = headers
            .get("X-Real-IP")
            .or_else(|| headers.get("X-Forwarded-For"))
            .and_then(|v| v.to_str().ok());
        let Some(raw) = raw else {
            return Ok(Self(None));
        };
        // X-Forwarded-For may be a comma-separated client,proxy list — take the
        // first (original client) entry.
        let first = raw.split(',').next().unwrap_or(raw).trim();
        let ip = IpAddr::from_str(first).map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Cannot convert proxy IP header: {err}"),
            )
                .into_response()
        })?;

        Ok(Self(Some(SocketAddr::new(ip, 0))))
    }
}
