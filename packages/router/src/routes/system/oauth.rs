use anyhow::Result;
use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::{ExtractIP, ExtractUserAgent};

#[tracing::instrument(skip_all)]
pub async fn oauth(
    ConnectInfo(native_ip): ConnectInfo<SocketAddr>,
    ExtractIP(ip): ExtractIP,
    ExtractUserAgent(user_agent): ExtractUserAgent,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ip = ip.unwrap_or(native_ip);

    // TODO: 三种 OAuth 更新方案

    Ok(().into_response())
}
