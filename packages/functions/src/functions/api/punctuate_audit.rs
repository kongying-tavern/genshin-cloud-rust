use anyhow::{anyhow, Result};
use chrono::Utc;

use sea_orm::{prelude::*, ActiveValue::Set, QueryFilter, QuerySelect};

use _database::{
    models::marker::marker as marker_model, models::marker::marker_item_link as mil_model, DB_CONN,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        marker::{MarkerEmptyResponse, MarkerListResponse, MarkerSingleResponse, MarkerVO},
        punctuate_audit::PunctuateAuditFilterRequest,
        wrapper::{CommonResponse, Pagination},
    },
};

/// 使用 `marker` 表作为审计后端：提供按 id、按筛选、按 id 列表和分页查询；
/// 审核通过/驳回会在 `marker.extra.audit` 写入简单记录以保留历史。
pub async fn do_get_id(_auth: AuthInfo, id: i64) -> Result<CommonResponse<MarkerSingleResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let m = marker_model::Entity::find_safety_by_id(id).one(db).await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let vo = MarkerVO {
        version: m.version,
        id: m.id,
        create_time: m.create_time.and_utc().timestamp_millis() as f64,
        update_time: m
            .update_time
            .map(|dt| dt.and_utc().timestamp_millis() as f64),
        creator_id: m.creator_id,
        updater_id: m.updater_id,
        del_flag: m.del_flag,
        marker_stamp: m.marker_stamp,
        marker_title: m.marker_title,
        position: m.position,
        content: m.content,
        picture: m.picture,
        marker_creator_id: m.marker_creator_id,
        picture_creator_id: m.picture_creator_id,
        video_path: m.video_path,
        refresh_time: m.refresh_time,
        hidden_flag: m.hidden_flag,
        extra: m.extra,
    };
    Ok(CommonResponse::new(Ok(MarkerSingleResponse { marker: vo })))
}

pub async fn do_get_list_by_info(
    _auth: AuthInfo,
    payload: PunctuateAuditFilterRequest,
) -> Result<CommonResponse<MarkerListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    // 如果提供了 item_id_list，则通过 marker_item_link 构建候选 marker id
    let mut ids: Option<Vec<i64>> = None;
    if let Some(item_ids) = payload.item_id_list {
        let links = mil_model::Entity::find_safety()
            .filter(mil_model::Column::ItemId.is_in(item_ids))
            .all(db)
            .await?;
        let set: std::collections::HashSet<i64> = links.into_iter().map(|l| l.marker_id).collect();
        let mut v: Vec<i64> = set.into_iter().collect();
        v.sort_unstable();
        ids = Some(v);
    }

    let mut query = marker_model::Entity::find_safety();
    // author_list 对应 creator_id
    if let Some(authors) = payload.author_list {
        query = query.filter(marker_model::Column::CreatorId.is_in(authors));
    }
    if let Some(v) = ids {
        if v.is_empty() {
            return Ok(CommonResponse::new(Ok(MarkerListResponse {
                total: 0,
                items: vec![],
            })));
        }
        query = query.filter(marker_model::Column::Id.is_in(v));
    }

    let items = query.all(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(MarkerVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
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
    Ok(CommonResponse::new(Ok(MarkerListResponse {
        total: arr.len(),
        items: arr,
    })))
}

pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    payload: Vec<i64>,
) -> Result<CommonResponse<MarkerListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    if payload.is_empty() {
        return Ok(CommonResponse::new(Ok(MarkerListResponse {
            total: 0,
            items: vec![],
        })));
    }
    let items = marker_model::Entity::find_safety()
        .filter(marker_model::Column::Id.is_in(payload))
        .all(db)
        .await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(MarkerVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
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
    Ok(CommonResponse::new(Ok(MarkerListResponse {
        total: arr.len(),
        items: arr,
    })))
}

pub async fn do_get_page_all(
    _auth: AuthInfo,
    payload: Pagination,
) -> Result<CommonResponse<MarkerListResponse>> {
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
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
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
    Ok(CommonResponse::new(Ok(MarkerListResponse {
        total: total as usize,
        items: arr,
    })))
}

pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let m = marker_model::Entity::find_safety_by_id(id).one(db).await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let mut am: marker_model::ActiveModel = m.into();
    am.del_flag = Set(true);
    marker_model::Entity::delete_safety(am).exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_pass(
    _auth: AuthInfo,
    payload: serde_json::Value,
) -> Result<CommonResponse<MarkerEmptyResponse>> {
    // 预期 payload: { "id": i64, "by": i64, "note": Option<String> }
    let db = &DB_CONN.wait().pg_conn;
    let id = payload
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or(anyhow!("id required"))?;
    let by = payload.get("by").and_then(|v| v.as_i64()).unwrap_or(0);
    let note = payload
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let m = marker_model::Entity::find_safety_by_id(id).one(db).await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let mut am: marker_model::ActiveModel = m.clone().into();

    // 将审核结果写入 extra.audit（追加到数组）。从模型的 extra 读取并更新
    let mut extra = m.extra.clone().unwrap_or(serde_json::json!({}));
    let mut audits = extra.get("audit").cloned().unwrap_or(serde_json::json!([]));
    let entry = serde_json::json!({"action": "pass", "by": by, "note": note, "time": Utc::now().to_rfc3339()});
    if audits.is_array() {
        audits.as_array_mut().unwrap().push(entry);
    } else {
        audits = serde_json::json!([entry]);
    }
    extra["audit"] = audits;
    am.extra = Set(Some(extra));

    marker_model::Entity::update_safety(am).exec(db).await?;
    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}

pub async fn do_reject(
    _auth: AuthInfo,
    payload: serde_json::Value,
) -> Result<CommonResponse<MarkerEmptyResponse>> {
    // 与 pass 类似，但标记为 reject
    let db = &DB_CONN.wait().pg_conn;
    let id = payload
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or(anyhow!("id required"))?;
    let by = payload.get("by").and_then(|v| v.as_i64()).unwrap_or(0);
    let note = payload
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let m = marker_model::Entity::find_safety_by_id(id).one(db).await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let mut am: marker_model::ActiveModel = m.clone().into();

    let mut extra = m.extra.clone().unwrap_or(serde_json::json!({}));
    let mut audits = extra.get("audit").cloned().unwrap_or(serde_json::json!([]));
    let entry = serde_json::json!({"action": "reject", "by": by, "note": note, "time": Utc::now().to_rfc3339()});
    if audits.is_array() {
        audits.as_array_mut().unwrap().push(entry);
    } else {
        audits = serde_json::json!([entry]);
    }
    extra["audit"] = audits;
    am.extra = Set(Some(extra));

    marker_model::Entity::update_safety(am).exec(db).await?;
    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}
