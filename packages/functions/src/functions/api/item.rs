use anyhow::Result;

use _utils::models::item::ItemUpdateData;
use _utils::{
    jwt::AuthInfo,
    models::{item::ItemAddRequest, item::ItemFilterRequest},
};

pub async fn do_update(
    _auth: AuthInfo,
    _edit_same: bool,
    _payload: Vec<ItemUpdateData>,
) -> Result<()> {
    let _ = (_auth, &_edit_same, &_payload);
    Ok(())
}

pub async fn do_get_list(
    _auth: AuthInfo,
    _payload: ItemFilterRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_join_type(_auth: AuthInfo, _type_id: i64, _payload: Vec<i64>) -> Result<()> {
    let _ = (_auth, &_type_id, &_payload);
    Ok(())
}

pub async fn do_get_list_by_id(_auth: AuthInfo, _payload: Vec<i64>) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}

pub async fn do_copy_to_area(_auth: AuthInfo, _area_id: i64, _payload: Vec<i64>) -> Result<i64> {
    let _ = (_auth, &_area_id, &_payload);
    Ok(0)
}

pub async fn do_add(_auth: AuthInfo, _payload: ItemAddRequest) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}
