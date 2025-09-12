use anyhow::Result;

use _utils::jwt::AuthInfo;
use _utils::models::notice::{NoticeAddRequest, NoticeListRequest, NoticeUpdateRequest};

pub async fn do_update_notice(_auth: AuthInfo, _payload: NoticeUpdateRequest) -> Result<()> {
    let _ = (_auth, &_payload);
    Ok(())
}

pub async fn do_get_notice_list(
    _auth: AuthInfo,
    _payload: NoticeListRequest,
) -> Result<serde_json::Value> {
    let _ = (_auth, &_payload);
    Ok(serde_json::json!({}))
}

pub async fn do_delete_notice(_auth: AuthInfo, _id: i64) -> Result<()> {
    Ok(())
}

pub async fn do_add_notice(_auth: AuthInfo, _payload: NoticeAddRequest) -> Result<i64> {
    let _ = (_auth, &_payload);
    Ok(0)
}
