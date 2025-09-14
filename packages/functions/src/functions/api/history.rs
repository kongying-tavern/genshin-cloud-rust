use anyhow::{anyhow, Result};

use chrono::{TimeZone, Utc};

use sea_orm::{prelude::*, QueryOrder, QuerySelect};

use _database::models::common::history as history_model;
use _database::DB_CONN;
use _utils::db_operations::SafeEntityTrait;
use _utils::{
    jwt::AuthInfo,
    models::{
        history::HistoryItemVO, history::HistoryListRequest, history::HistoryListResponse,
        wrapper::CommonResponse,
    },
};

pub async fn do_get_list(
    _auth: AuthInfo,
    payload: HistoryListRequest,
) -> Result<CommonResponse<HistoryListResponse>> {
    // 构建安全查询
    let mut query = history_model::Entity::find_safety();

    if let Some(start_ts) = payload.create_time_start {
        let ndt = Utc
            .timestamp_opt(start_ts as i64, 0)
            .single()
            .ok_or_else(|| anyhow!("Invalid start timestamp"))?
            .naive_utc();
        query = query.filter(history_model::Column::CreateTime.gte(ndt));
    }
    if let Some(end_ts) = payload.create_time_end {
        let ndt = Utc
            .timestamp_opt(end_ts as i64, 0)
            .single()
            .ok_or_else(|| anyhow!("Invalid end timestamp"))?
            .naive_utc();
        query = query.filter(history_model::Column::CreateTime.lte(ndt));
    }

    if let Some(creator_s) = payload.creator_id {
        // 支持逗号分隔的多 id 或单个 id 字符串
        let ids: Vec<i64> = creator_s
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect();
        if !ids.is_empty() {
            if ids.len() == 1 {
                query = query.filter(history_model::Column::CreatorId.eq(ids[0]));
            } else {
                query = query.filter(history_model::Column::CreatorId.is_in(ids));
            }
        }
    }

    if let Some(edit_type) = payload.edit_type {
        query = query.filter(history_model::Column::EditType.eq(edit_type));
    }

    if let Some(ids) = payload.id {
        if !ids.is_empty() {
            query = query.filter(history_model::Column::TId.is_in(ids));
        }
    }

    // 排序
    if let Some(sorts) = payload.sort {
        for s in sorts {
            match s {
                _utils::models::history::HistorySort::UpdateTimeAsc => {
                    query = query.order_by_asc(history_model::Column::UpdateTime)
                }
                _utils::models::history::HistorySort::UpdateTimeDesc => {
                    query = query.order_by_desc(history_model::Column::UpdateTime)
                }
            }
        }
    } else {
        query = query.order_by_desc(history_model::Column::UpdateTime);
    }

    // 统计总数
    let total = query.clone().count(&DB_CONN.wait().pg_conn).await?;

    // 分页
    let mut select = query;
    if let Some(current) = payload.page.current {
        if let Some(size) = payload.page.size {
            let offset = (current.saturating_sub(1) as u64).saturating_mul(size as u64);
            select = select.limit(size as u64).offset(offset as u64);
        }
    }

    let items = select.all(&DB_CONN.wait().pg_conn).await?;
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
            t_id: it.t_id,
            history_type: it.history_type,
            edit_type: it.edit_type,
            content: it.content,
        });
    }

    Ok(CommonResponse::new(Ok(HistoryListResponse {
        total: total as usize,
        items: arr,
    })))
}
