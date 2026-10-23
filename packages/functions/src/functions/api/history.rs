use anyhow::Result;

use chrono::{TimeZone, Utc};

use sea_orm::{QueryOrder, QuerySelect, prelude::*};

use std::collections::HashSet;

use _database::DatabaseConnectionMap;
use _database::models::common::history as history_model;
use _utils::db_operations::SafeEntityTrait;
use _utils::{
    errors::DomainError,
    jwt::AuthInfo,
    models::{
        history::HistoryItemVO, history::HistoryListRequest, history::HistoryListResponse,
        wrapper::CommonResponse,
    },
};

/// 写入一条历史记录（Java `MarkerService.saveHistory` /
/// `ItemService.saveHistoryItem` 的等价物）。
///
/// 此前 Rust 侧只读不写：history 表空转导致 `/api/history/get/list` 恒空、
/// score 生成无数据源。content 为实体**变更前**状态的 camelCase DTO JSON
/// （Java 语义：更新/删除记录的是旧值，tweak 记录被批量改写的原值集）。
pub(crate) async fn record(
    db: &sea_orm::DatabaseConnection,
    ty: _utils::types::HistoryOperationType,
    edit: _utils::types::HistoryEditType,
    t_id: i64,
    content: String,
    creator_id: Option<i64>,
) -> Result<()> {
    use sea_orm::ActiveValue::{NotSet, Set};

    let now = Utc::now().naive_utc();
    let md5_hex = format!("{:x}", md5::compute(content.as_bytes()));
    history_model::Entity::insert(history_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：历史行由操作触发，create/update 同刻
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(creator_id),
        updater_id: Set(creator_id),
        del_flag: Set(false),

        content: Set(content),
        md5: Set(Some(md5_hex)),
        t_id: Set(t_id),
        history_type: Set(Some(ty)),
        ipv4: Set(None),
        edit_type: Set(edit),
    })
    .exec(db)
    .await?;
    Ok(())
}

pub async fn do_get_list(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    payload: HistoryListRequest,
) -> Result<CommonResponse<HistoryListResponse>> {
    // Java 网关矩阵把 /api/history/get/** 归 MAP_USER 级：匿名/访客不可读，
    // 真实登录用户可读（并非 Manager 级）——在函数层挡匿名即对齐该门槛。
    auth.require_non_anonymous()?;
    // 构建安全查询
    let mut query = history_model::Entity::find_safety();

    if let Some(start_ts) = payload.create_time_start {
        let ndt = Utc
            .timestamp_opt(start_ts as i64, 0)
            .single()
            .ok_or_else(|| DomainError::Business("Invalid start timestamp".into()))?
            .naive_utc();
        query = query.filter(history_model::Column::CreateTime.gte(ndt));
    }
    if let Some(end_ts) = payload.create_time_end {
        let ndt = Utc
            .timestamp_opt(end_ts as i64, 0)
            .single()
            .ok_or_else(|| DomainError::Business("Invalid end timestamp".into()))?
            .naive_utc();
        query = query.filter(history_model::Column::CreateTime.lte(ndt));
    }

    if let Some(creator_id) = payload.creator_id {
        query = query.filter(history_model::Column::CreatorId.eq(creator_id));
    }

    if let Some(edit_type) = payload.edit_type {
        query = query.filter(history_model::Column::EditType.eq(edit_type));
    }

    // 请求体 `type`（HistoryListRequest.request_type，数字 1..=4）按 history 表的 type 列过滤
    if let Some(ty) = payload.request_type {
        query = query.filter(history_model::Column::HistoryType.eq(ty));
    }

    if let Some(ids) = payload.id
        && !ids.is_empty()
    {
        query = query.filter(history_model::Column::TId.is_in(ids));
    }

    // 排序
    if let Some(sorts) = payload.sort {
        for s in sorts {
            match s {
                _utils::models::history::HistorySort::UpdateTimeAsc => {
                    query = query.order_by_asc(history_model::Column::UpdateTime)
                },
                _utils::models::history::HistorySort::UpdateTimeDesc => {
                    query = query.order_by_desc(history_model::Column::UpdateTime)
                },
            }
        }
    } else {
        query = query.order_by_desc(history_model::Column::UpdateTime);
    }

    // 统计总数
    let total = query.clone().count(&db.pg_conn).await?;

    // 分页（apply a default limit when page params are missing to avoid
    // loading the entire history table — it can have 400K+ rows)
    let mut select = query;
    let current = payload.page.current.unwrap_or(1);
    let size = payload.page.size.unwrap_or(20).min(200);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size as u64);
    select = select.limit(size as u64).offset(offset);

    let items = select.all(&db.pg_conn).await?;
    let creator_ids: HashSet<i64> = items.iter().filter_map(|it| it.creator_id).collect();
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(HistoryItemVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: it.creator_id,
            updater_id: it.updater_id,
            del_flag: it.del_flag,
            md5: it.md5,
            ipv4: it.ipv4,
            t_id: it.t_id,
            history_type: it.history_type,
            edit_type: it.edit_type,
            content: it.content,
        });
    }

    let users = super::sys_user_map(&db.pg_conn, &creator_ids).await?;
    Ok(CommonResponse::new(Ok(HistoryListResponse {
        total: total as usize,
        size: size as usize,
        items: arr,
    }))
    .with_users(users))
}
