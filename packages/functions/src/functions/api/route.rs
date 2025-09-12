use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{
        route::{RouteAddRequest, RouteSearchRequest, RouteUpdateRequest},
        wrapper::Pagination,
    },
};

pub async fn do_add(_auth: AuthInfo, _payload: RouteAddRequest) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}

pub async fn do_update(_auth: AuthInfo, _payload: RouteUpdateRequest) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_get_page(_auth: AuthInfo, _payload: Pagination) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_search(
    _auth: AuthInfo,
    _payload: RouteSearchRequest,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_list_by_id(_auth: AuthInfo, _payload: Vec<f64>) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}
