//! User device business logic — mirrors Java `SysUserDeviceService`.

use anyhow::{Result, anyhow};
use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::system::sys_user_device as device_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{SysUserDeviceVo, wrapper::CommonResponse},
    types::DeviceSort,
};

/// List user devices with optional filtering.
pub async fn do_list(
    _auth: AuthInfo,
    user_id: Option<i64>,
    device_id: Option<String>,
    status: Option<i32>,
    sort: Option<Vec<DeviceSort>>,
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

    // 排序：显式枚举映射，变体重命名会变成编译错误而非静默忽略排序键。
    if let Some(sorts) = sort {
        use sea_orm::QueryOrder;
        for s in sorts {
            let (column, desc) = match s {
                DeviceSort::DeviceId => (device_model::Column::DeviceId, false),
                DeviceSort::DeviceIdReverse => (device_model::Column::DeviceId, true),
                DeviceSort::Id => (device_model::Column::Id, false),
                DeviceSort::IdReverse => (device_model::Column::Id, true),
                DeviceSort::Ipv4 => (device_model::Column::Ipv4, false),
                DeviceSort::Ipv4Reverse => (device_model::Column::Ipv4, true),
                DeviceSort::LastLoginTime => (device_model::Column::LastLoginTime, false),
                DeviceSort::LastLoginTimeReverse => (device_model::Column::LastLoginTime, true),
                DeviceSort::Status => (device_model::Column::Status, false),
                DeviceSort::StatusReverse => (device_model::Column::Status, true),
                DeviceSort::UpdateTime => (device_model::Column::UpdateTime, false),
                DeviceSort::UpdateTimeReverse => (device_model::Column::UpdateTime, true),
            };
            query = if desc {
                query.order_by(column, sea_orm::Order::Desc)
            } else {
                query.order_by(column, sea_orm::Order::Asc)
            };
        }
    }

    let total = query.clone().count(db).await?;
    let offset = current.saturating_sub(1).saturating_mul(size);
    let items = query.limit(size).offset(offset).all(db).await?;
    let record: Vec<SysUserDeviceVo> = items
        .into_iter()
        .map(|d| SysUserDeviceVo {
            id: d.id,
            create_time: d.create_time.and_utc().timestamp_millis() as f64,
            update_time: d.update_time.map(|t| t.and_utc().timestamp_millis() as f64),
            user_id: d.user_id,
            device_id: d.device_id,
            ipv4: d.ipv4,
            status: d.status,
            last_login_time: d
                .last_login_time
                .map(|t| t.and_utc().timestamp_millis() as f64),
        })
        .collect();

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "record": record,
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
