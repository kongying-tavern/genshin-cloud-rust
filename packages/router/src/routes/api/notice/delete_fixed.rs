use axum::extract::Path;
use _utils::models::wrapper::ServiceResponse;
use crate::middlewares::ExtractAuthInfo;

/// 删除公告
#[tracing::instrument(skip_all)]
pub async fn delete_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(notice_id): Path<i64>,
) -> ServiceResponse<serde_json::Value> {
    // TODO: 实现公告删除逻辑
    ServiceResponse::success(serde_json::json!({
        "message": "公告删除成功"
    }))
}
