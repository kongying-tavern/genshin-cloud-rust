use anyhow::Result;

use _utils::jwt::AuthInfo;
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

pub async fn do_all_list_bin_md5(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_all_list_bin(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_all_graph_bin_md5(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_all_graph_bin(_auth: AuthInfo, _payload: serde_json::Value) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
