//! Action log business logic — mirrors Java `SysActionLogService`.

use anyhow::Result;
use sea_orm::{QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::system::sys_action_log as log_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

/// List action logs with optional filtering by user_id / action.
pub async fn do_list(
    _auth: AuthInfo,
    user_id: Option<i64>,
    _action: Option<i64>,
    size: u64,
    current: u64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = log_model::Entity::find_safety();

    if let Some(uid) = user_id {
        query = query.filter(log_model::Column::UserId.eq(uid));
    }

    let total = query.clone().count(db).await?;
    let offset = current.saturating_sub(1).saturating_mul(size);
    let items = query.limit(size).offset(offset).all(db).await?;

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "list": items,
    }))))
}
