use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::history::HistoryListRequest,
};

pub async fn do_get_list(_auth: AuthInfo, _payload: HistoryListRequest) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}
