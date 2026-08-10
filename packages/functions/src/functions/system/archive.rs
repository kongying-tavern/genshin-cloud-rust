//! User archive (save slot) business logic — mirrors Java `SysUserArchiveService`.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, QueryFilter, QueryOrder, QuerySelect,
    prelude::*,
};

use _database::{DB_CONN, models::system::sys_user_archive as archive_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{ArchiveSlotVo, wrapper::CommonResponse},
};

/// 把实体行转换成槽位 VO：`data` 列存的是存档 JSON 文本，直接作为 `archive` 字段；
/// 兼容历史数据（data 为非字符串 JSON 值）时退化为重新序列化。
fn entity_to_slot_vo(a: archive_model::Model) -> ArchiveSlotVo {
    ArchiveSlotVo {
        slot_index: a.slot_index,
        time: a.create_time.and_utc().timestamp_millis() as f64,
        archive: a
            .data
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| serde_json::to_string(&a.data).unwrap_or_default()),
    }
}

/// 从任意 JSON body 中提取存档文本：
/// 前端直接上传 `JSON.stringify({Data_KYJG, Time_KYJG, Preference})` 的存档体，
/// 若带有 `archive` 字段（`{time, archive, historyIndex}` 包装体）则取该字段，
/// 否则把整个请求体序列化作为存档文本。
fn extract_archive(body: &serde_json::Value) -> String {
    body.get("archive")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| serde_json::to_string(body).unwrap_or_default())
}

/// Get the latest archive for a given slot index.
pub async fn do_get_last(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<Option<ArchiveSlotVo>>> {
    let db = &DB_CONN.wait().pg_conn;
    let archive = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .one(db)
        .await?;
    Ok(CommonResponse::new(Ok(archive.map(entity_to_slot_vo))))
}

/// Get all history archives for a given slot index.
pub async fn do_get_history(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<Vec<serde_json::Value>>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .all(db)
        .await?;
    let record: Vec<serde_json::Value> = items
        .into_iter()
        .map(|a| serde_json::to_value(entity_to_slot_vo(a)).unwrap_or_default())
        .collect();
    Ok(CommonResponse::new(Ok(record)))
}

/// Get all history archives across all slots for the user.
/// 返回按槽位分组的 `SysArchiveSlotVo` 结构：元素为
/// `{slotIndex, time, updateTime, archive: [SysArchiveVo]}`（archive 为数组，
/// 每个元素是 {time, archive, historyIndex}），对齐前端 `stores/archive.ts`
/// `data.forEach(...)` + `historyArchives.map(...)` 的遍历契约。
pub async fn do_get_all_history(
    _auth: AuthInfo,
    user_id: i64,
) -> Result<CommonResponse<Vec<serde_json::Value>>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .order_by_asc(archive_model::Column::SlotIndex)
        .order_by_asc(archive_model::Column::CreateTime)
        .all(db)
        .await?;

    let mut record: Vec<serde_json::Value> = Vec::new();
    // 当前分组的槽位状态：存档列表 + 最新时间
    let mut group_slot: Option<i32> = None;
    let mut group_time: f64 = 0.0;
    let mut group_archives: Vec<serde_json::Value> = Vec::new();
    let mut history_index: i64 = 0;
    for a in items {
        let slot = a.slot_index;
        if group_slot.is_some() && group_slot != Some(slot) {
            record.push(serde_json::json!({
                "slotIndex": group_slot,
                "time": group_time,
                "updateTime": group_time,
                "archive": group_archives,
            }));
            group_archives = Vec::new();
            history_index = 0;
        }
        group_slot = Some(slot);
        let ts = a.create_time.and_utc().timestamp_millis() as f64;
        if ts > group_time {
            group_time = ts;
        }
        let archive = a
            .data
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| serde_json::to_string(&a.data).unwrap_or_default());
        // data 本身是该次快照的存档 JSON：能解析成对象就放进历史列表，否则空条目兜底
        group_archives.push(serde_json::json!({
            "time": ts,
            "archive": archive,
            "historyIndex": history_index,
        }));
        history_index += 1;
    }
    if group_slot.is_some() {
        record.push(serde_json::json!({
            "slotIndex": group_slot,
            "time": group_time,
            "updateTime": group_time,
            "archive": group_archives,
        }));
    }
    Ok(CommonResponse::new(Ok(record)))
}

/// 槽位历史条数上限：每用户每槽位最多保留最新 `MAX_HISTORY_PER_SLOT` 条，
/// 超出部分在 `do_save` 写入后立即软删最旧记录（防止存档表随前端反复
/// 保存无限膨胀，同时保证历史列表接口返回规模可控）。
const MAX_HISTORY_PER_SLOT: u64 = 20;

/// 写入后清理：按 create_time desc 保留最新 `MAX_HISTORY_PER_SLOT` 条，
/// 其余软删。
async fn prune_slot_history(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    slot_index: i64,
) -> Result<()> {
    let keep_ids: Vec<i64> = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .limit(MAX_HISTORY_PER_SLOT)
        .all(db)
        .await?
        .into_iter()
        .map(|m| m.id)
        .collect();
    if keep_ids.is_empty() {
        return Ok(());
    }
    // 批量软删（避免逐条 find+delete 的 N+1 往返）
    archive_model::Entity::update_many()
        .col_expr(
            archive_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .filter(archive_model::Column::Id.is_not_in(keep_ids))
        .exec(db)
        .await?;
    Ok(())
}

/// Save (put) an archive to a slot.
/// 请求体为任意 JSON：前端直接上传存档 JSON 字符串；兼容 `{time, archive, historyIndex}` 包装体。
pub async fn do_save(
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
    name: Option<String>,
    body: serde_json::Value,
) -> Result<CommonResponse<serde_json::Value>> {
    // 写操作：匿名（client_credentials，uid=0）一律拒绝——否则所有匿名
    // 客户端共享 uid=0 档案，互相覆盖/读取。
    auth.require_non_anonymous()?;
    // 槽位列类型为 i32：路由层已限定 0..=4，此处兜底做收窄校验，不做截断
    let slot_index: i32 =
        i32::try_from(slot_index).map_err(|_| anyhow!("slot_index out of range"))?;
    let db = &DB_CONN.wait().pg_conn;
    let archive = extract_archive(&body);
    // create_time 由服务端定，不信任客户端传入的 time（防止时间戳伪造/脏数据）
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
        data: Set(serde_json::Value::String(archive)),
    };
    let res = archive_model::Entity::insert(am).exec(db).await?;
    // 写入后按上限清理最旧历史
    prune_slot_history(db, user_id, slot_index.into()).await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "id": res.last_insert_id
    }))))
}

/// Rename an archive slot.
///
/// 注意：按 id 操作，**未校验存档归属**（user_id == 请求者）。当前无路由
/// 接线（router 只用 do_rename_by_slot），一旦接线即构成 IDOR——任意登录
/// 用户可按 id 改他人存档。启用前必须先按 `auth.info.id` 过滤归属。
pub async fn do_rename(auth: AuthInfo, id: i64, name: String) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
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
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
    new_name: String,
) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
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
///
/// 注意：按 id 操作，**未校验存档归属**（user_id == 请求者）。当前无路由
/// 接线（router 只用 do_restore_slot），一旦接线即构成 IDOR——任意登录
/// 用户可按 id 读取他人存档数据。启用前必须先按 `auth.info.id` 过滤归属。
pub async fn do_restore(auth: AuthInfo, id: i64) -> Result<CommonResponse<serde_json::Value>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let a = archive_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Archive not found"))?;
    Ok(CommonResponse::new(Ok(a.data)))
}

/// 恢复为上次存档：删除该槽位最新一条（按 create_time desc 取第一条软删），
/// 然后返回剩余最新一条（结构同 `do_get_last`）。
pub async fn do_restore_slot(
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<Option<ArchiveSlotVo>>> {
    // 恢复即删除最新一条历史，属写操作
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let latest = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .one(db)
        .await?;
    let Some(latest) = latest else {
        return Ok(CommonResponse::new(Ok(None)));
    };
    archive_model::Entity::delete_safety(latest.into())?
        .exec(db)
        .await?;
    let next = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_desc(archive_model::Column::CreateTime)
        .one(db)
        .await?;
    Ok(CommonResponse::new(Ok(next.map(entity_to_slot_vo))))
}

/// Delete an archive slot (soft-delete every archive in the slot).
pub async fn do_delete_slot(
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    // 批量软删（避免逐条 find+delete 的 N+1 往返）
    archive_model::Entity::update_many()
        .col_expr(
            archive_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .exec(db)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}

/// Delete an archive slot (soft delete).
///
/// 注意：按 id 操作，**未校验存档归属**（user_id == 请求者）。当前无路由
/// 接线（router 只用 do_delete_slot），一旦接线即构成 IDOR——任意登录
/// 用户可按 id 删除他人存档。启用前必须先按 `auth.info.id` 过滤归属。
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
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
