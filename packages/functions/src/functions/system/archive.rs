//! User archive (save slot) business logic — mirrors Java `SysUserArchiveService`.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QueryOrder,
    prelude::*,
};

use _database::{DB_CONN, models::system::sys_user_archive as archive_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

/// Get the latest archive for a given slot index.
pub async fn do_get_last(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i32,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let archive = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .one(db)
        .await?;
    Ok(CommonResponse::new(Ok(serde_json::to_value(archive)?)))
}

/// Get all history archives for a given slot index.
pub async fn do_get_history(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i32,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .all(db)
        .await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": items.len(),
        "list": items,
    }))))
}

/// Get all history archives across all slots for the user.
pub async fn do_get_all_history(
    _auth: AuthInfo,
    user_id: i64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .order_by_desc(archive_model::Column::CreateTime)
        .all(db)
        .await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": items.len(),
        "list": items,
    }))))
}

/// Save (put) an archive to a slot.
pub async fn do_save(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i32,
    name: Option<String>,
    data: serde_json::Value,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let am = archive_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(user_id)),
        updater_id: Set(None),
        del_flag: Set(false),
        name: Set(name),
        slot_index: Set(slot_index),
        user_id: Set(user_id),
        data: Set(data),
    };
    let res = archive_model::Entity::insert(am).exec(db).await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "id": res.last_insert_id
    }))))
}

/// Rename an archive slot.
pub async fn do_rename(_auth: AuthInfo, id: i64, name: String) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    let a = archive_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Archive not found"))?;
    let mut am: archive_model::ActiveModel = a.into();
    am.name = Set(Some(name));
    archive_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}

/// Rename an archive slot (renames the latest archive in the slot).
pub async fn do_rename_by_slot(
    user_id: i64,
    slot_index: i32,
    new_name: String,
) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    let a = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Archive not found"))?;
    let mut am: archive_model::ActiveModel = a.into();
    am.name = Set(Some(new_name));
    archive_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}

/// Restore from an archive (return the archived data).
pub async fn do_restore(_auth: AuthInfo, id: i64) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let a = archive_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Archive not found"))?;
    Ok(CommonResponse::new(Ok(a.data)))
}

/// Delete an archive slot (soft-delete every archive in the slot).
pub async fn do_delete_slot(user_id: i64, slot_index: i32) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .all(db)
        .await?;
    for a in items {
        archive_model::Entity::delete_safety(a.into())?
            .exec(db)
            .await?;
    }
    Ok(CommonResponse::new(Ok(())))
}

/// Delete an archive slot (soft delete).
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    let a = archive_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Archive not found"))?;
    archive_model::Entity::delete_safety(a.into())?
        .exec(db)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}
