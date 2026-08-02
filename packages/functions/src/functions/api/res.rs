use _utils::jwt::AuthInfo;
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};
use anyhow::Result;

pub async fn do_get() -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_upload_image(
    auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
