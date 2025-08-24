use axum::extract::Multipart;
use _utils::models::wrapper::ServiceResponse;
use crate::middlewares::ExtractAuthInfo;

/// 上传图片
#[tracing::instrument(skip_all)]
pub async fn upload_image(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    mut multipart: Multipart,
) -> ServiceResponse<serde_json::Value> {
    // TODO: 实现图片上传逻辑
    ServiceResponse::success(serde_json::json!({
        "url": "https://example.com/image.jpg",
        "message": "图片上传成功"
    }))
}
