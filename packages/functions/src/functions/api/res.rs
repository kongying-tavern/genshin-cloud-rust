use anyhow::Result;

use _utils::jwt::AuthInfo;

pub async fn do_upload_image(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}
