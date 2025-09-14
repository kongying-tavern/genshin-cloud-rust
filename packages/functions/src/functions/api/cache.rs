use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{common::EmptyResponse, wrapper::CommonResponse},
};

pub async fn do_delete_notice_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete_marker_link_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete_marker_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete_item_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete_icon_tag_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete_common_item_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete_area_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
