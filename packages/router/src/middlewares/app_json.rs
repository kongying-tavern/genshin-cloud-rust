//! 与 axum::Json 相同的请求体提取，但反序列化失败（422）时
//! 返回标准 JSON 错误体，而不是 axum 默认的纯文本。

use crate::middlewares::{ApiError, api_error};
use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
};
use serde::de::DeserializeOwned;

/// JSON 请求体提取器（错误统一 JSON 化）。
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(AppJson(value)),
            Err(rejection) => {
                let message = rejection.body_text();
                Err(api_error(StatusCode::UNPROCESSABLE_ENTITY, &message))
            },
        }
    }
}
