use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{
        item::ItemFilterRequest,
        marker_link::{
            MarkerLinkDeleteRequest, MarkerLinkGraphRequest, MarkerLinkListRequest, MarkerLinkage,
        },
    },
};

pub async fn do_link(_auth: AuthInfo, _payload: Vec<MarkerLinkage>) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_list(
    _auth: AuthInfo,
    _payload: MarkerLinkListRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_get_graph(
    _auth: AuthInfo,
    _payload: MarkerLinkGraphRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _payload: MarkerLinkDeleteRequest) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}
