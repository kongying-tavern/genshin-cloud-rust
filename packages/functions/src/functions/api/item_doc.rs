use anyhow::Result;

use _utils::jwt::AuthInfo;
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

pub async fn do_list_page_bin_md5(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_list_page_bin(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
