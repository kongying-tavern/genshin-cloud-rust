use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::wrapper::Pagination,
};

pub async fn do_get_list(_auth: AuthInfo, _payload: Pagination) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_single(_auth: AuthInfo, _id: i64) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_add(_auth: AuthInfo, _payload: Vec<i64>) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}

pub async fn do_update(_auth: AuthInfo, _payload: serde_json::Value) -> Result<()> {
    Ok(())
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}
