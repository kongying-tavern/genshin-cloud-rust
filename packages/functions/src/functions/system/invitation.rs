//! User invitation business logic — mirrors Java `SysUserInvitationService`.

use anyhow::{Result, anyhow};
use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::system::sys_user_invitation as inv_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

/// List invitations with optional filtering by code / username.
pub async fn do_list(
    _auth: AuthInfo,
    code: Option<String>,
    username: Option<String>,
    size: u64,
    current: u64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = inv_model::Entity::find_safety();

    if let Some(c) = code {
        query = query.filter(inv_model::Column::Code.eq(c));
    }
    if let Some(u) = username {
        query = query.filter(inv_model::Column::Username.eq(u));
    }

    let total = query.clone().count(db).await?;
    let offset = current.saturating_sub(1).saturating_mul(size);
    let items = query.limit(size).offset(offset).all(db).await?;

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "list": items,
    }))))
}

/// Update an invitation by code (e.g. change role_id or remark).
pub async fn do_update(
    _auth: AuthInfo,
    code: String,
    role_id: Option<i64>,
    remark: Option<String>,
) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;

    let inv = inv_model::Entity::find_safety()
        .filter(inv_model::Column::Code.eq(&code))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invitation not found"))?;
    let mut am: inv_model::ActiveModel = inv.into();
    if let Some(r) = role_id {
        // role_id is stored as an enum; set via numeric value
        am.role_id = Set(Some(match r {
            0 => _utils::types::SystemUserRole::Admin,
            1 => _utils::types::SystemUserRole::MapNeigui,
            2 => _utils::types::SystemUserRole::MapManager,
            3 => _utils::types::SystemUserRole::MapPunctuate,
            4 => _utils::types::SystemUserRole::MapUser,
            5 => _utils::types::SystemUserRole::Visitor,
            _ => return Err(anyhow!("Invalid role id")),
        }));
    }
    if let Some(rm) = remark {
        am.remark = Set(Some(rm));
    }
    inv_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}

/// Check invitation info by code.
pub async fn do_info(auth: AuthInfo, code: String) -> Result<CommonResponse<serde_json::Value>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let inv = inv_model::Entity::find_safety()
        .filter(inv_model::Column::Code.eq(code))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invitation code not found"))?;
    Ok(CommonResponse::new(Ok(serde_json::to_value(inv)?)))
}

/// Consume (use) an invitation code — marks it as used by deleting it.
pub async fn do_consume(auth: AuthInfo, code: String) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let inv = inv_model::Entity::find_safety()
        .filter(inv_model::Column::Code.eq(code))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invitation code not found"))?;
    inv_model::Entity::delete_safety(inv.into())?
        .exec(db)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}

/// Delete an invitation by id (soft delete).
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    let inv = inv_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invitation not found"))?;
    inv_model::Entity::delete_safety(inv.into())?
        .exec(db)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}
