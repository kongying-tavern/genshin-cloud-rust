use anyhow::Result;

use _utils::jwt::AuthInfo;

pub async fn do_all_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_all_bin(_auth: AuthInfo, _payload: serde_json::Value) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}
