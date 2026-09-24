//! User archive (save slot) business logic — mirrors Java `SysUserArchiveService`.
//!
//! 存储模型（Java `SysUserArchiveSlotDto`）：同一 `user_id + slot_index` 只保留
//! 一行；`data` 列为历史存档数组（默认 `[]`），**最新在前**，元素形如
//! `{ "time": <毫秒时间戳>, "archive": <存档文本> }`；每次保存向数组头部插入
//! 一条，超出上限挤掉最旧一条。
//!
//! 脏数据兼容：历史数据存在同 `user_id + slot_index` 多行的情况，取值时按
//! **最新更新时间（update_time）+ 最大 ID 兜底**取第一条。

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, QueryFilter, QueryOrder,
    prelude::*,
};

use _database::{DB_CONN, models::system::sys_user_archive as archive_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{SysArchiveSlotVo, SysArchiveVo, wrapper::CommonResponse},
};

/// 每槽位历史条数上限（Java `saveArchive`：槽位已满时挤掉最旧一次备份）
const MAX_HISTORY_PER_SLOT: usize = 5;

/// 存档历史条目（`data` 数组元素），最新在前。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArchiveEntry {
    /// 存档时间（毫秒级数字时间戳）
    pub time: i64,
    /// 存档文本（前端 `JSON.stringify` 的存档 JSON）
    pub archive: String,
}

/// 解析 `data` 列为历史条目列表（保持存储顺序，最新在前）：
/// - 业务默认值 `[]`（null / 非数组按空列表处理）；
/// - 兼容历史脏数据：`time` 为毫秒数字或数字字符串，无法解析按 0；
///   `archive` 为非字符串 JSON 时重新序列化为文本。
fn parse_entries(data: &serde_json::Value) -> Vec<ArchiveEntry> {
    let mut ret = Vec::new();
    let Some(arr) = data.as_array() else {
        // 兼容旧 Rust 版本的写入形态（data 直接是存档 JSON 字符串）：
        // 视为单条无时间戳的历史条目，保证旧数据仍可读出。
        if let Some(text) = data.as_str() {
            ret.push(ArchiveEntry {
                time: 0,
                archive: text.to_string(),
            });
        }
        return ret;
    };
    for v in arr {
        let time = v
            .get("time")
            .and_then(|t| match t {
                serde_json::Value::Number(n) => n.as_i64(),
                serde_json::Value::String(s) => s.trim().parse::<i64>().ok(),
                _ => None,
            })
            .unwrap_or(0);
        let archive = match v.get("archive") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(other) if !other.is_null() => serde_json::to_string(other).unwrap_or_default(),
            _ => String::new(),
        };
        ret.push(ArchiveEntry { time, archive });
    }
    ret
}

/// 历史条目 → 前端 VO（`historyIndex` 从 1 起，Java `getVo(index)`）。
fn entry_to_vo(entry: &ArchiveEntry, history_index: i64) -> SysArchiveVo {
    SysArchiveVo {
        time: entry.time as f64,
        archive: entry.archive.clone(),
        history_index,
    }
}

/// 实体行 → 槽位 VO（`SysArchiveSlotVo`；update_time 为空回退 create_time）。
fn row_to_slot_vo(a: archive_model::Model) -> SysArchiveSlotVo {
    let entries = parse_entries(&a.data);
    SysArchiveSlotVo {
        version: a.version,
        id: a.id,
        name: a.name,
        slot_index: a.slot_index,
        create_time: a.create_time.and_utc().timestamp_millis() as f64,
        update_time: Some(
            a.update_time
                .unwrap_or(a.create_time)
                .and_utc()
                .timestamp_millis() as f64,
        ),
        archive: entries
            .iter()
            .enumerate()
            .map(|(i, e)| entry_to_vo(e, (i + 1) as i64))
            .collect(),
    }
}

/// 定位槽位行（脏数据兼容）：同 `user_id + slot_index` 多行时，按
/// update_time 降序（NULL 视为最旧）+ 最大 ID 兜底取第一条。
async fn find_slot_row(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    slot_index: i32,
) -> Result<Option<archive_model::Model>> {
    Ok(archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .order_by_with_nulls(
            archive_model::Column::UpdateTime,
            sea_orm::Order::Desc,
            sea_orm::sea_query::NullOrdering::Last,
        )
        .order_by_desc(archive_model::Column::Id)
        .one(db)
        .await?)
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
/// 最新存档即 1 号历史记录（`data` 数组首条）。
pub async fn do_get_last(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<Option<SysArchiveVo>>> {
    let db = &DB_CONN.wait().pg_conn;
    let slot_index = i32::try_from(slot_index).map_err(|_| anyhow!("slot_index out of range"))?;
    let row = find_slot_row(db, user_id, slot_index).await?;
    let Some(row) = row else {
        return Ok(CommonResponse::new(Ok(None)));
    };
    let latest = parse_entries(&row.data).first().map(|e| entry_to_vo(e, 1));
    Ok(CommonResponse::new(Ok(latest)))
}

/// Get all history archives for a given slot index.
/// 返回单个 `SysArchiveSlotVo`（Java `getSlot`；槽位不存在报错同 Java 语义）。
pub async fn do_get_history(
    _auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<SysArchiveSlotVo>> {
    let db = &DB_CONN.wait().pg_conn;
    let slot_index = i32::try_from(slot_index).map_err(|_| anyhow!("slot_index out of range"))?;
    let row = find_slot_row(db, user_id, slot_index)
        .await?
        .ok_or_else(|| anyhow!("槽位不存在"))?;
    Ok(CommonResponse::new(Ok(row_to_slot_vo(row))))
}

/// Get all history archives across all slots for the user.
/// 按 slotIndex 升序返回 `SysArchiveSlotVo` 列表；脏数据同槽位多行时
/// 每槽位只保留取值规则（最新 update_time + 最大 ID）命中的那一行。
pub async fn do_get_all_history(
    _auth: AuthInfo,
    user_id: i64,
) -> Result<CommonResponse<Vec<SysArchiveSlotVo>>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(user_id))
        .order_by_asc(archive_model::Column::SlotIndex)
        .order_by_with_nulls(
            archive_model::Column::UpdateTime,
            sea_orm::Order::Desc,
            sea_orm::sea_query::NullOrdering::Last,
        )
        .order_by_desc(archive_model::Column::Id)
        .all(db)
        .await?;

    let mut record: Vec<SysArchiveSlotVo> = Vec::new();
    for a in items {
        // 同槽位多行（脏数据）：只保留排序命中的第一条
        if record
            .last()
            .is_some_and(|vo| vo.slot_index == a.slot_index)
        {
            continue;
        }
        record.push(row_to_slot_vo(a));
    }
    Ok(CommonResponse::new(Ok(record)))
}

/// Save (put) an archive to a slot.
///
/// 同一 `user_id + slot_index` 最多保留一行：
/// - 首次保存：插入槽位行，`data = [entry]`；
/// - 再次保存：向 `data` 数组头部插入新条目（Java `saveArchive` 的
///   `add(0, ...)`），与最新一条一致时不写入并返回 false（RBoolean 契约），
///   超过上限挤掉最旧一条。
///
/// 请求体为任意 JSON：前端直接上传存档 JSON 字符串；兼容
/// `{time, archive, historyIndex}` 包装体。`name`（PUT 端点）非空时顺带更新槽位名。
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
    let now = Utc::now().naive_utc();
    // time 字段使用毫秒级数字时间戳（服务端时间，不信任客户端传入）
    let entry = ArchiveEntry {
        time: now.and_utc().timestamp_millis(),
        archive: archive.clone(),
    };

    match find_slot_row(db, user_id, slot_index).await? {
        Some(row) => {
            let mut entries = parse_entries(&row.data);
            // 幂等去重（Java saveArchive / 前端 RBoolean 契约）：与该槽位
            // 最新一条存档内容一致时不写入，返回 false。
            if entries.first().is_some_and(|e| e.archive == archive) {
                return Ok(CommonResponse::new(Ok(serde_json::json!(false))));
            }
            // 最新在前：向数组头部插入，超上限挤掉最旧
            entries.insert(0, entry);
            entries.truncate(MAX_HISTORY_PER_SLOT);

            let mut am: archive_model::ActiveModel = row.into();
            am.data = Set(serde_json::to_value(&entries)?);
            if let Some(n) = name.filter(|n| !n.is_empty()) {
                am.name = Set(Some(n));
            }
            // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
            am.updater_id = Set(Some(user_id));
            am.update_time = Set(Some(now));
            archive_model::Entity::update_safety(am)?.exec(db).await?;
        },
        None => {
            let am = archive_model::ActiveModel {
                version: Set(1),
                id: NotSet,
                // 审计字段：新增时 create/update 两组全部设置
                create_time: Set(now),
                update_time: Set(Some(now)),
                creator_id: Set(Some(user_id)),
                updater_id: Set(Some(user_id)),
                del_flag: Set(false),
                name: Set(name),
                slot_index: Set(slot_index),
                user_id: Set(user_id),
                data: Set(serde_json::to_value(&[entry])?),
            };
            archive_model::Entity::insert(am).exec(db).await?;
        },
    }
    // RBoolean 契约：成功写入返回 true
    Ok(CommonResponse::new(Ok(serde_json::json!(true))))
}

/// Rename an archive slot.
pub async fn do_rename_by_slot(
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
    new_name: String,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let slot_index = i32::try_from(slot_index).map_err(|_| anyhow!("slot_index out of range"))?;
    let row = find_slot_row(db, user_id, slot_index)
        .await?
        .ok_or_else(|| anyhow!("槽位不存在"))?;
    let mut am: archive_model::ActiveModel = row.into();
    am.name = Set(Some(new_name));
    // 审计字段：修改时设置 update 组
    am.updater_id = Set(Some(auth.info.id));
    archive_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(true)))
}

/// Restore from an archive (drop the newest history entry).
/// Java `restoreArchive`/`restoreHistory`：删除最近一次存档并返回被删除的
/// 那条（historyIndex=1）；存档为空时报「存档为空，无历史存档」。
pub async fn do_restore_slot(
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<SysArchiveVo>> {
    // 恢复即删除最新一条历史，属写操作
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let slot_index = i32::try_from(slot_index).map_err(|_| anyhow!("slot_index out of range"))?;
    let row = find_slot_row(db, user_id, slot_index)
        .await?
        .ok_or_else(|| anyhow!("存档为空，无历史存档"))?;
    let mut entries = parse_entries(&row.data);
    if entries.is_empty() {
        return Err(anyhow!("存档为空，无历史存档"));
    }
    let removed = entries.remove(0);

    let now = Utc::now().naive_utc();
    let mut am: archive_model::ActiveModel = row.into();
    am.data = Set(serde_json::to_value(&entries)?);
    // 审计字段：修改时设置 update 组
    am.updater_id = Set(Some(auth.info.id));
    am.update_time = Set(Some(now));
    archive_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(entry_to_vo(&removed, 1))))
}

/// Delete an archive slot (soft-delete the slot row(s)).
/// 脏数据同槽位多行时一并清理；槽位不存在报「槽位不存在」（Java 同文案）。
pub async fn do_delete_slot(
    auth: AuthInfo,
    user_id: i64,
    slot_index: i64,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let slot_index = i32::try_from(slot_index).map_err(|_| anyhow!("slot_index out of range"))?;
    if find_slot_row(db, user_id, slot_index).await?.is_none() {
        return Err(anyhow!("槽位不存在"));
    }
    // 批量软删（含脏数据多行；update_many 不走实体钩子，手动补 update 组）
    let now = Utc::now().naive_utc();
    archive_model::Entity::update_many()
        .col_expr(
            archive_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .col_expr(
            archive_model::Column::UpdateTime,
            sea_orm::sea_query::Expr::value(now),
        )
        .col_expr(
            archive_model::Column::UpdaterId,
            sea_orm::sea_query::Expr::value(auth.info.id),
        )
        .filter(archive_model::Column::UserId.eq(user_id))
        .filter(archive_model::Column::SlotIndex.eq(slot_index))
        .exec(db)
        .await?;
    Ok(CommonResponse::new(Ok(true)))
}
