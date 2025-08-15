use anyhow::Result;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};

#[allow(dead_code)]
pub struct ExtractUserAgent(pub String);

impl<S> FromRequestParts<S> for ExtractUserAgent
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = parts.headers.clone();

        match headers.get("User-Agent") {
            Some(user_agent) => {
                let user_agent = user_agent.to_str().map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Cannot parse User-Agent: {}", err),
                    )
                        .into_response()
                })?;

                Ok(Self(user_agent.to_string()))
            }
            None => Ok(Self("".to_string())),
        }
    }
}
