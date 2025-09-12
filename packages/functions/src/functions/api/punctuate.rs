use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{punctuate::PunctuateData, wrapper::Pagination},
};
pub async fn do_update(_auth: AuthInfo, _payload: PunctuateData) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_submit(_auth: AuthInfo, _payload: PunctuateData) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}

pub async fn do_get_page(_auth: AuthInfo, _payload: Pagination) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_page_by_author(
    _auth: AuthInfo,
    _author_id: i64,
    _payload: Pagination,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_author_id, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_push(_auth: AuthInfo, _payload: serde_json::Value) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_delete(_auth: AuthInfo, _payload: serde_json::Value) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}
