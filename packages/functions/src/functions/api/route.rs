use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        route::{RouteAddRequest, RouteSearchRequest, RouteUpdateRequest, RouteEmptyResponse},
        wrapper::{Pagination, CommonResponse},
    },
};

pub async fn do_add(_auth: AuthInfo, _payload: RouteAddRequest) -> Result<i64> {
    Ok(0)
}

pub async fn do_update(_auth: AuthInfo, _payload: RouteUpdateRequest) -> Result<CommonResponse<EmptyResponse>> {
    let _ = _payload;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_get_page(_auth: AuthInfo, _payload: Pagination) -> Result<CommonResponse<RouteEmptyResponse>> {
    Ok(CommonResponse::new(Ok(RouteEmptyResponse {})))
}

pub async fn do_get_search(
    _auth: AuthInfo,
    _payload: RouteSearchRequest,
) -> Result<RouteEmptyResponse> {
    Ok(RouteEmptyResponse {})
}

pub async fn do_get_list_by_id(_auth: AuthInfo, _payload: Vec<f64>) -> Result<CommonResponse<RouteEmptyResponse>> {
    Ok(CommonResponse::new(Ok(RouteEmptyResponse {})))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let _ = _id;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
