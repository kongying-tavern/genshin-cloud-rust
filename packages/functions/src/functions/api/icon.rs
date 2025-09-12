use anyhow::Result;

use _utils::jwt::AuthInfo;

pub async fn do_add(_auth: AuthInfo, _payload: serde_json::Value) -> Result<i64> {
    Ok(0)
}

pub async fn do_list(_auth: AuthInfo, _payload: serde_json::Value) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_single(_auth: AuthInfo, _id: i64) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}

pub async fn do_update(_auth: AuthInfo, _payload: serde_json::Value) -> Result<()> {
    Ok(())
}
