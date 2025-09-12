use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{marker::MarkerFilterRequest, wrapper::Pagination, marker::{MarkerTweakRequest, MarkerAddRequest, MarkerUpdateData}},
};

pub async fn do_tweak(_auth: AuthInfo, _payload: MarkerTweakRequest) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_add_single(_auth: AuthInfo, _payload: MarkerAddRequest) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}

pub async fn do_update_single(_auth: AuthInfo, _payload: MarkerUpdateData) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_get_id(_auth: AuthInfo, _payload: MarkerFilterRequest) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_list_by_info(
    _auth: AuthInfo,
    _payload: MarkerFilterRequest,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    _payload: Vec<i64>,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_page(
    _auth: AuthInfo,
    _payload: Pagination,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}
