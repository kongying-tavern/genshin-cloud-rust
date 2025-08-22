use anyhow::Result;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use _utils::{
    jwt::{verify_token, AuthInfo},
    models::CommonResponse,
};

pub struct ExtractAuthInfo(pub Option<AuthInfo>);

impl<S> FromRequestParts<S> for ExtractAuthInfo
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(bearer) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        {
            let token = bearer.token().to_string();
            let claims = verify_token(&token).await.map_err(|err| {
                (
                    StatusCode::UNAUTHORIZED,
                    serde_json::to_string(&CommonResponse::<()>::new(Err(err)))
                        .expect("Failed to serialize error response"),
                )
                    .into_response()
            })?;

            return Ok(Self(Some(AuthInfo {
                token,
                user_id: claims.sub,
                created_at: claims.iat,
                expires_at: claims.exp,
            })));
        }

        Ok(Self(None))
    }
}
