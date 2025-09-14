use anyhow::Result;
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};
use _utils::jwt::AuthInfo;

pub async fn do_get() -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_upload_image(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
