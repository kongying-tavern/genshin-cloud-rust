use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::item_type::{ItemTypeAddRequest, ItemTypeListRequest, ItemTypeUpdateData},
};

pub async fn do_update(_auth: AuthInfo, _payload: ItemTypeUpdateData) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_move_to_target(
    _auth: AuthInfo,
    _target_type_id: i64,
    _payload: Vec<i64>,
) -> Result<()> {
    let _ = (_auth, &_target_type_id, &_payload);
    Ok(())
}

pub async fn do_get_list(
    _auth: AuthInfo,
    _self_flag: bool,
    _payload: ItemTypeListRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_self_flag, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_list_all(_auth: AuthInfo) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}

pub async fn do_add(_auth: AuthInfo, _payload: ItemTypeAddRequest) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}
