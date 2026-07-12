//! User device business logic — mirrors Java `SysUserDeviceService`.

use anyhow::{Result, anyhow};
use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::system::sys_user_device as device_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

/// List user devices with optional filtering.
pub async fn do_list(
    _auth: AuthInfo,
    user_id: Option<i64>,
    device_id: Option<String>,
    status: Option<i32>,
    size: u64,
    current: u64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = device_model::Entity::find_safety();

    if let Some(uid) = user_id {
        query = query.filter(device_model::Column::UserId.eq(uid));
    }
    if let Some(did) = device_id {
        query = query.filter(device_model::Column::DeviceId.eq(did));
    }
    if let Some(s) = status {
        query = query.filter(device_model::Column::Status.eq(s));
    }

    let total = query.clone().count(db).await?;
    let offset = current.saturating_sub(1).saturating_mul(size);
    let items = query.limit(size).offset(offset).all(db).await?;

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "list": items,
    }))))
}

/// Update device status (e.g. block/unblock a device).
pub async fn do_update(_auth: AuthInfo, id: i64, status: i32) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;

    let d = device_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Device not found"))?;
    let mut am: device_model::ActiveModel = d.into();
    am.status = Set(status);
    device_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}
