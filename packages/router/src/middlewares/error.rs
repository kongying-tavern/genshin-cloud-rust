//! 统一的 JSON 错误响应。
//!
//! 所有 handler 与中间件的失败路径都返回与 CommonResponse 序列化一致
//! 的 JSON 错误体，前端才能稳定解析并呈现错误信息。

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::Value;

use _utils::models::CommonResponse;

/// 标准错误类型：状态码 + JSON 错误体（与 CommonResponse 序列化一致）。
pub struct ApiError {
    pub status: StatusCode,
    pub body: Json<Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, self.body).into_response()
    }
}

impl From<(StatusCode, String)> for ApiError {
    fn from((status, message): (StatusCode, String)) -> Self {
        api_error(status, &message)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("{e}"))
    }
}

/// 构造标准 JSON 错误响应。
pub fn api_error(status: StatusCode, message: &str) -> ApiError {
    let body = CommonResponse::<()>::new(Err(anyhow::anyhow!(message.to_string())))
        .with_status(status.as_u16());
    ApiError {
        status,
        body: Json(serde_json::to_value(&body).unwrap_or_else(|_| {
            serde_json::json!({
                "error": true,
                "errorStatus": status.as_u16(),
                "message": message,
                "data": null,
                "users": {}
            })
        })),
    }
}
