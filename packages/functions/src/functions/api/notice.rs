use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryOrder,
    prelude::*,
};

use std::collections::HashSet;

use _database::{
    DB_CONN,
    models::common::notice::{self as notice_model, ChannelWrapper},
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        notice::{
            NoticeAddRequest, NoticeChannel, NoticeListRequest, NoticeListResponse,
            NoticeUpdateRequest, NoticeVO,
        },
        wrapper::CommonResponse,
    },
};

/// 转义 LIKE 通配符（% _ \），防止输入被当作模糊匹配通配符放大（PG 默认 ESCAPE 为反斜杠）。
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// 解析有效期字段：接受毫秒数字或 ISO/普通时间字符串，解析失败回退 `now`。
/// `None` 或 JSON `null` 表示前端传空（保持原值/NULL），不回退 `now`。
fn parse_valid_time(
    value: Option<&serde_json::Value>,
    now: chrono::NaiveDateTime,
) -> Option<chrono::NaiveDateTime> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => Some(if let Some(ms) = v.as_f64() {
            chrono::DateTime::from_timestamp_millis(ms as i64)
                .map(|dt| dt.naive_utc())
                .unwrap_or(now)
        } else if let Some(s) = v.as_str() {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.naive_utc())
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                .unwrap_or(now)
        } else {
            now
        }),
    }
}

pub async fn do_update_notice(
    auth: AuthInfo,
    payload: NoticeUpdateRequest,
) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let n = notice_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let n = n.ok_or(anyhow!("Notice not found"))?;
    let mut am: notice_model::ActiveModel = n.into();

    let now = Utc::now().naive_utc();
    // NoticeUpdateRequest 包含具体字段：全量写回（含频道/排序/有效期）
    am.title = Set(payload.title);
    am.content = Set(Some(payload.content));
    am.channel = Set(ChannelWrapper(
        payload
            .channel
            .iter()
            .map(|c| {
                serde_json::to_string(c)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string()
            })
            .collect(),
    ));
    am.sort_index = Set(payload.sort_index.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
    am.valid_time_start = Set(parse_valid_time(payload.valid_time_start.as_ref(), now));
    am.valid_time_end = Set(parse_valid_time(payload.valid_time_end.as_ref(), now));

    notice_model::Entity::update_safety(am)?.exec(db).await?;
    super::super::ws::ws_broadcast("NoticeUpdated", serde_json::json!(payload.id));
    Ok(CommonResponse::new(Ok(())))
}

pub async fn do_get_notice_list(
    _auth: AuthInfo,
    payload: NoticeListRequest,
) -> Result<CommonResponse<NoticeListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let mut query = notice_model::Entity::find_safety();
    if let Some(title) = payload.title {
        query =
            query.filter(notice_model::Column::Title.like(format!("%{}%", escape_like(&title))));
    }
    // 公告量小：全量取回后在内存做 channel/有效期过滤（对齐前端参数，
    // jsonb 数组的 SQL 过滤跨方言繁琐且不值得）。
    let mut rows = query
        .order_by_desc(notice_model::Column::SortIndex)
        .all(db)
        .await?;

    if let Some(channels) = payload.channels {
        let wanted: Vec<String> = channels
            .iter()
            .map(|c| {
                serde_json::to_string(c)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string()
            })
            .collect();
        rows.retain(|n| {
            let chans: Vec<String> =
                serde_json::from_value(serde_json::to_value(&n.channel).unwrap_or_default())
                    .unwrap_or_default();
            chans.iter().any(|c| wanted.contains(c))
        });
    }
    if payload.get_valid.unwrap_or(false) {
        let now = Utc::now().naive_utc();
        rows.retain(|n| {
            n.valid_time_start.is_none_or(|s| s <= now) && n.valid_time_end.is_none_or(|e| e >= now)
        });
    }

    let total = rows.len();
    let creator_ids: HashSet<i64> = rows.iter().filter_map(|n| n.creator_id).collect();
    let size = payload.page.size.unwrap_or(10).min(200) as usize;
    let current = payload.page.current.unwrap_or(1) as usize;
    let offset = (current.saturating_sub(1)) * size;
    let items = rows.into_iter().skip(offset).take(size);

    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        // map channel wrapper (Vec<String>) to NoticeChannel enum where possible
        let mut channels: Vec<NoticeChannel> = Vec::new();
        if let serde_json::Value::Array(arr_val) = serde_json::to_value(&it.channel)? {
            for v in arr_val {
                if let Some(s) = v.as_str() {
                    match s {
                        "APPLICATION" => channels.push(NoticeChannel::Application),
                        "CLIENT_APP" => channels.push(NoticeChannel::ClientApp),
                        "CLIENT_PC" => channels.push(NoticeChannel::ClientPc),
                        "COMMON" => channels.push(NoticeChannel::Common),
                        "DADIAN" => channels.push(NoticeChannel::Dadian),
                        "DASHBOARD" => channels.push(NoticeChannel::Dashboard),
                        "TIANLI" => channels.push(NoticeChannel::Tianli),
                        "WEB" => channels.push(NoticeChannel::Web),
                        _ => {},
                    }
                }
            }
        }

        arr.push(NoticeVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: it.creator_id,
            updater_id: it.updater_id,
            title: it.title,
            content: it.content,
            channels,
            sort_index: it.sort_index as i64,
            valid_time_start: it
                .valid_time_start
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            valid_time_end: it
                .valid_time_end
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
        });
    }
    let payload = NoticeListResponse {
        total: total as i64,
        items: arr,
        size: Some(size as i64),
    };
    let users = super::sys_user_map(db, &creator_ids).await?;
    Ok(CommonResponse::new(Ok(payload)).with_users(users))
}

pub async fn do_delete_notice(auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let n = notice_model::Entity::find_safety_by_id(id).one(db).await?;
    let n = n.ok_or(anyhow!("Notice not found"))?;
    let mut am: notice_model::ActiveModel = n.into();
    am.del_flag = Set(true);
    notice_model::Entity::delete_safety(am)?.exec(db).await?;
    super::super::ws::ws_broadcast("NoticeDeleted", serde_json::json!(id));
    Ok(CommonResponse::new(Ok(())))
}

pub async fn do_add_notice(
    auth: AuthInfo,
    payload: NoticeAddRequest,
) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();
    let active = notice_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),

        title: Set(payload.title),
        content: Set(Some(payload.content)),
        channel: Set(ChannelWrapper(
            payload
                .channel
                .iter()
                .map(|c| {
                    serde_json::to_string(c)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string()
                })
                .collect(),
        )),
        sort_index: Set(payload.sort_index.clamp(i32::MIN as i64, i32::MAX as i64) as i32),
        valid_time_start: Set(parse_valid_time(payload.valid_time_start.as_ref(), now)),
        valid_time_end: Set(parse_valid_time(payload.valid_time_end.as_ref(), now)),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    super::super::ws::ws_broadcast("NoticeAdded", serde_json::json!(res.id));
    Ok(CommonResponse::new(Ok(res.id)))
}
