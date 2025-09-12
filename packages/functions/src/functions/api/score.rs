use _utils::jwt::AuthInfo;
use _utils::models::score::{ScoreDataRequest, ScoreGenerateRequest};
use anyhow::Result;

pub async fn do_generate_score(
    _auth: AuthInfo,
    _payload: ScoreGenerateRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    // TODO: implement
    Ok(serde_json::json!({}))
}

pub async fn do_get_score_data(
    _auth: AuthInfo,
    _payload: ScoreDataRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    // TODO: implement
    Ok(serde_json::json!({}))
}
