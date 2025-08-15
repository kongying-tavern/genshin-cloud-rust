use anyhow::Result;
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
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
        let headers = parts.headers.clone();

        match headers.get("X-Real-IP") {
            Some(ip) => {
                let ip = IpAddr::from_str(ip.to_str().map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Cannot parse X-Real-IP: {}", err),
                    )
                        .into_response()
                })?)
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Cannot convert X-Real-IP: {}", err),
                    )
                        .into_response()
                })?;

                Ok(Self(Some(SocketAddr::new(ip, 0))))
            }
            None => Ok(Self(None)),
        }
    }
}
