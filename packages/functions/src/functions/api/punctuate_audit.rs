use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{wrapper::Pagination, punctuate_audit::PunctuateAuditFilterRequest},
};

pub async fn do_get_id(_auth: AuthInfo, _id: i64) -> Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub async fn do_get_list_by_info(
    _auth: AuthInfo,
    _payload: PunctuateAuditFilterRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    _payload: Vec<i64>,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_page_all(
    _auth: AuthInfo,
    _payload: Pagination,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}

pub async fn do_pass(_auth: AuthInfo, _payload: serde_json::Value) -> Result<()> {
    Ok(())
}

pub async fn do_reject(_auth: AuthInfo, _payload: serde_json::Value) -> Result<()> {
    Ok(())
}
