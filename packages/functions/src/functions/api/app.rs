//! Application-level utilities.
//!
//! Mirrors Java `AppController`: `trigger/update` broadcasts an app-update
//! event (Java uses SocketIO). This backend has no socket layer, so the
//! equivalent effect is to flush every BinaryMD5 cache — clients refetch the
//! archives on their next poll.

use anyhow::Result;

use _utils::{jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc;

/// `POST /app/trigger/update` — flush all BinaryMD5 caches and report success.
pub async fn do_trigger_update(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all().await;
    super::super::ws::ws_broadcast("AppUpdated", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}
