use anyhow::Result;
use chrono::Utc;

use sea_orm::{prelude::*, ActiveValue::Set, QueryFilter, QuerySelect};

use _database::{models::marker::marker as marker_model, DB_CONN};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        punctuate::PunctuateData,
        marker::{MarkerAddResponse, MarkerEmptyResponse, MarkerVO, MarkerListResponse},
        wrapper::{CommonResponse, Pagination},
    },
};

/// 使用 marker 表作为打点后台实现
pub async fn do_update(_auth: AuthInfo, payload: PunctuateData) -> Result<CommonResponse<MarkerEmptyResponse>> {
    // 如果 payload 中带有原有点位 id，则视为更新对应 marker 的额外字段与状态
    if let Some(original_id) = payload.original_marker_id {
        let id = original_id as i64;
        let db = &DB_CONN.wait().pg_conn;
        if let Some(model) = marker_model::Entity::find_safety_by_id(id).one(db).await? {
            // 更新 extra、content、position、picture、video_path 等可变字段
            let mut am: marker_model::ActiveModel = model.into();
            am.content = Set(payload.content.unwrap_or_default());
            am.position = Set(payload.position);
            am.picture = Set(payload.picture);
            am.video_path = Set(payload.video_path);
            if let Some(extra) = payload.extra {
                let json = serde_json::to_value(extra)?;
                am.extra = Set(Some(json));
            }

            marker_model::Entity::update_safety(am).exec(db).await?;
        }
    }

    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}

pub async fn do_submit(_auth: AuthInfo, payload: PunctuateData) -> Result<CommonResponse<MarkerAddResponse>> {
    let now = Utc::now().naive_utc();
    let active = marker_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(payload.author)),
        updater_id: Set(None),
        del_flag: Set(false),

        marker_stamp: Set(None),
        marker_title: Set(Some(payload.marker_title)),
        position: Set(payload.position),
        content: Set(payload.content.unwrap_or_default()),
        picture: Set(payload.picture),
        marker_creator_id: Set(payload.marker_creator_id),
        picture_creator_id: Set(payload.picture_creator_id),
        video_path: Set(payload.video_path),
        refresh_time: Set(payload.refresh_time.unwrap_or(0)),
        hidden_flag: Set(payload.hidden_flag),
        extra: Set(payload
            .extra
            .map(|m| serde_json::to_value(m).unwrap_or(serde_json::json!({})))),
        ..Default::default()
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(MarkerAddResponse { id: res.id })))
}

pub async fn do_get_page(_auth: AuthInfo, payload: Pagination) -> Result<CommonResponse<MarkerListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let query = marker_model::Entity::find_safety();
    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(MarkerVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it.update_time.map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: it.creator_id,
            updater_id: it.updater_id,
            del_flag: it.del_flag,
            marker_stamp: it.marker_stamp,
            marker_title: it.marker_title,
            position: it.position,
            content: it.content,
            picture: it.picture,
            marker_creator_id: it.marker_creator_id,
            picture_creator_id: it.picture_creator_id,
            video_path: it.video_path,
            refresh_time: it.refresh_time,
            hidden_flag: it.hidden_flag,
            extra: it.extra,
        });
    }
    Ok(CommonResponse::new(Ok(MarkerListResponse { total: total as usize, items: arr })))
}

pub async fn do_get_page_by_author(
    _auth: AuthInfo,
    author_id: i64,
    payload: Pagination,
) -> Result<CommonResponse<MarkerListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let query =
        marker_model::Entity::find_safety().filter(marker_model::Column::CreatorId.eq(author_id));
    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(MarkerVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it.update_time.map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: it.creator_id,
            updater_id: it.updater_id,
            del_flag: it.del_flag,
            marker_stamp: it.marker_stamp,
            marker_title: it.marker_title,
            position: it.position,
            content: it.content,
            picture: it.picture,
            marker_creator_id: it.marker_creator_id,
            picture_creator_id: it.picture_creator_id,
            video_path: it.video_path,
            refresh_time: it.refresh_time,
            hidden_flag: it.hidden_flag,
            extra: it.extra,
        });
    }
    Ok(CommonResponse::new(Ok(MarkerListResponse { total: total as usize, items: arr })))
}

pub async fn do_push(_auth: AuthInfo, payload: serde_json::Value) -> Result<CommonResponse<MarkerEmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    // 如果 payload 包含明确的 marker id，则更新其 extra（向后兼容）
    if let Some(id) = payload.get("id").and_then(|v| v.as_i64()) {
        if let Some(model) = marker_model::Entity::find_safety_by_id(id).one(db).await? {
            let mut am: marker_model::ActiveModel = model.into();
            if let Some(extra) = payload.get("extra") {
                am.extra = Set(Some(extra.clone()));
            }
            marker_model::Entity::update_safety(am).exec(db).await?;
            return Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})));
        }
    }

    // 否则如果 payload 包含 author_id（router 使用该字段），则视为提交待审：
    // 查找该作者创建的 markers，并将 submit 审计条目追加到 extra.audit
    if let Some(author_id) = payload.get("author_id").and_then(|v| v.as_i64()) {
        let markers = marker_model::Entity::find_safety()
            .filter(marker_model::Column::MarkerCreatorId.eq(author_id))
            .all(db)
            .await?;

        for m in markers.into_iter() {
            let mut am: marker_model::ActiveModel = m.clone().into();

            let mut extra = m.extra.clone().unwrap_or(serde_json::json!({}));
            let mut audits = extra.get("audit").cloned().unwrap_or(serde_json::json!([]));
            let entry = serde_json::json!({
                "action": "submit",
                "by": author_id,
                "time": Utc::now().to_rfc3339()
            });
            if audits.is_array() {
                // 避免添加重复的连续 submit 条目
                let arr = audits.as_array_mut().unwrap();
                let need_push = match arr.last() {
                    Some(last) => last.get("action").and_then(|v| v.as_str()) != Some("submit"),
                    None => true,
                };
                if need_push {
                    arr.push(entry);
                }
            } else {
                audits = serde_json::json!([entry]);
            }
            extra["audit"] = audits;
            am.extra = Set(Some(extra));

            marker_model::Entity::update_safety(am).exec(db).await?;
        }
    }

    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}

pub async fn do_delete(
    _auth: AuthInfo,
    payload: serde_json::Value,
) -> Result<CommonResponse<MarkerEmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    // 支持按 id 删除 {"id": i64}
    if let Some(id) = payload.get("id").and_then(|v| v.as_i64()) {
        marker_model::Entity::delete_safety_by_id(id)
            .exec(db)
            .await?;
    }

    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}
