//! Action log business logic — mirrors Java `SysActionLogService`.

use anyhow::Result;
use sea_orm::{QueryFilter, QueryOrder, QuerySelect, prelude::*};

use _database::{DB_CONN, models::system::sys_action_log as log_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{SysActionLogVo, wrapper::CommonResponse},
    types::SystemActionLogAction,
};

/// 把动作枚举序列化为字符串（如 "LOGIN"）。
fn action_to_str(action: SystemActionLogAction) -> String {
    serde_json::to_value(action)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// List action logs with optional filtering by user_id / action.
#[allow(clippy::too_many_arguments)]
pub async fn do_list(
    _auth: AuthInfo,
    user_id: Option<i64>,
    action: Option<i64>,
    device_id: Option<String>,
    ipv4: Option<String>,
    is_error: Option<bool>,
    sort: Option<Vec<String>>,
    size: u64,
    current: u64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = log_model::Entity::find_safety();

    if let Some(uid) = user_id {
        query = query.filter(log_model::Column::UserId.eq(uid));
    }
    if let Some(action) = action {
        // ActionLogAction is a fieldless enum: Login = 0 (see types/action_log.rs).
        let enum_val = match action {
            0 => SystemActionLogAction::Login,
            other => {
                return Ok(CommonResponse::new(Ok(serde_json::json!({
                    "total": 0,
                    "record": [],
                    "note": format!("unknown action filter: {other}"),
                }))));
            },
        };
        query = query.filter(log_model::Column::Action.eq(enum_val));
    }
    if let Some(did) = device_id
        && !did.is_empty()
    {
        query = query.filter(log_model::Column::DeviceId.contains(did));
    }
    if let Some(ip) = ipv4
        && !ip.is_empty()
    {
        query = query.filter(log_model::Column::Ipv4.contains(ip));
    }
    if let Some(err) = is_error {
        query = query.filter(log_model::Column::IsError.eq(err));
    }

    // 排序：wire 字符串（"createTime+" / "createTime-" 等）→ 列 + 方向。
    if let Some(sorts) = sort {
        for s in sorts {
            let (column, desc) = match s.as_str() {
                "createTime+" => (log_model::Column::CreateTime, false),
                "createTime-" => (log_model::Column::CreateTime, true),
                "deviceId+" => (log_model::Column::DeviceId, false),
                "deviceId-" => (log_model::Column::DeviceId, true),
                "id+" => (log_model::Column::Id, false),
                "id-" => (log_model::Column::Id, true),
                "ipv4+" => (log_model::Column::Ipv4, false),
                "ipv4-" => (log_model::Column::Ipv4, true),
                "isError+" => (log_model::Column::IsError, false),
                "isError-" => (log_model::Column::IsError, true),
                "updateTime+" => (log_model::Column::UpdateTime, false),
                "updateTime-" => (log_model::Column::UpdateTime, true),
                _ => continue,
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
    let record: Vec<SysActionLogVo> = items
        .into_iter()
        .map(|l| SysActionLogVo {
            id: l.id,
            create_time: l.create_time.and_utc().timestamp_millis() as f64,
            update_time: l.update_time.map(|t| t.and_utc().timestamp_millis() as f64),
            user_id: l.user_id,
            ipv4: l.ipv4,
            device_id: l.device_id,
            action: action_to_str(l.action),
            is_error: l.is_error,
            extra_data: l
                .extra_data
                .map(|e| serde_json::to_value(e).unwrap_or_default()),
        })
        .collect();

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "record": record,
    }))))
}
