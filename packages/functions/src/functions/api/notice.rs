use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryOrder,
    prelude::*,
};

use _database::{DB_CONN, models::common::notice as notice_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        notice::{
            NoticeAddRequest, NoticeAddResponse, NoticeChannel, NoticeListRequest,
            NoticeListResponse, NoticeUpdateRequest, NoticeVO,
        },
        wrapper::CommonResponse,
    },
};

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

    // NoticeUpdateRequest 包含具体字段
    am.title = Set(payload.title);
    am.content = Set(Some(payload.content));

    notice_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}

pub async fn do_get_notice_list(
    _auth: AuthInfo,
    payload: NoticeListRequest,
) -> Result<CommonResponse<NoticeListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let mut query = notice_model::Entity::find_safety();
    if let Some(title) = payload.title {
        query = query.filter(notice_model::Column::Title.like(format!("%{}%", title)));
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
    let size = payload.page.size.unwrap_or(10) as usize;
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
            id: it.id,
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
    };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_delete_notice(auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let n = notice_model::Entity::find_safety_by_id(id).one(db).await?;
    let n = n.ok_or(anyhow!("Notice not found"))?;
    let mut am: notice_model::ActiveModel = n.into();
    am.del_flag = Set(true);
    notice_model::Entity::delete_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(())))
}

pub async fn do_add_notice(
    auth: AuthInfo,
    payload: NoticeAddRequest,
) -> Result<CommonResponse<NoticeAddResponse>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();
    let active = notice_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        title: Set(payload.title),
        content: Set(Some(payload.content)),
        ..Default::default()
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(NoticeAddResponse { id: res.id })))
}
