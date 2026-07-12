use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{
    DB_CONN, models::marker::marker as marker_model, models::marker::marker_punctuate as mp_model,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{common::EmptyResponse, wrapper::CommonResponse},
    types::{MarkerPunctuateMethodType, MarkerPunctuateStatus},
};

/// 按 punctuate_id 查询单条打点审核信息
pub async fn do_get_id(
    _auth: AuthInfo,
    punctuate_id: i64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let m = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(punctuate_id))
        .one(db)
        .await?;
    let m = m.ok_or_else(|| anyhow!("打点信息不存在"))?;
    Ok(CommonResponse::new(Ok(serde_json::to_value(m)?)))
}

/// 分页查询所有待审核的打点（status = Reviewing）
pub async fn do_get_page_all(
    _auth: AuthInfo,
    payload: _utils::models::Pagination,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let query = mp_model::Entity::find_safety()
        .filter(mp_model::Column::Status.eq(MarkerPunctuateStatus::Reviewing));
    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "list": items,
    }))))
}

/// 按提交者列表查询待审核打点
pub async fn do_get_list_by_authors(
    _auth: AuthInfo,
    authors: Vec<i64>,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = mp_model::Entity::find_safety()
        .filter(mp_model::Column::Status.eq(MarkerPunctuateStatus::Reviewing))
        .filter(mp_model::Column::Author.is_in(authors))
        .all(db)
        .await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": items.len(),
        "list": items,
    }))))
}

/// 按 punctuate_id 列表批量查询
pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    punctuate_ids: Vec<i64>,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let items = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.is_in(punctuate_ids))
        .all(db)
        .await?;
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": items.len(),
        "list": items,
    }))))
}

/// 审核通过：将 Reviewing 的打点按 method_type 晋升为正式 marker。
///
/// 对应 Java `PunctuateAuditService.passPunctuate`：
/// - Added：插入新 marker，删除 punctuate 记录
/// - Modified：更新 original_marker_id 对应的 marker，删除 punctuate 记录
/// - Deleted：软删除 original_marker_id 对应的 marker，删除 punctuate 记录
pub async fn do_pass(
    _auth: AuthInfo,
    punctuate_id: i64,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let mp = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(punctuate_id))
        .filter(mp_model::Column::Status.eq(MarkerPunctuateStatus::Reviewing))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("无打点相关信息，或该打点不在审核中状态"))?;

    let result_id = match mp.method_type {
        MarkerPunctuateMethodType::Added => {
            // 新增：将 punctuate 数据写入 marker 表
            let am = marker_model::ActiveModel {
                version: Set(0),
                id: Set(0),
                create_time: Set(now),
                update_time: Set(None),
                creator_id: Set(Some(mp.author)),
                updater_id: Set(None),
                del_flag: Set(false),
                marker_stamp: Set(None),
                marker_title: Set(mp.marker_title),
                position: Set(mp.position),
                content: Set(mp.content),
                picture: Set(mp.picture),
                marker_creator_id: Set(mp.marker_creator_id),
                picture_creator_id: Set(mp.picture_creator_id),
                video_path: Set(mp.video_path),
                refresh_time: Set(mp.refresh_time),
                hidden_flag: Set(mp.hidden_flag),
                extra: Set(mp.extra),
            };
            let res = marker_model::Entity::insert(am).exec(db).await?;
            // 删除 punctuate 记录（硬删除——它已完成使命）
            mp_model::Entity::delete_by_id(mp.id).exec(db).await?;
            res.last_insert_id
        },
        MarkerPunctuateMethodType::Modified => {
            // 修改：更新 original_marker_id 对应的 marker
            let orig_id = mp
                .original_marker_id
                .ok_or_else(|| anyhow!("无法找到修改点位的原始id"))?;

            let old = marker_model::Entity::find_safety_by_id(orig_id)
                .one(db)
                .await?
                .ok_or_else(|| anyhow!("无法找到原始id对应的原始点位"))?;

            let mut am: marker_model::ActiveModel = old.into();
            // 用 punctuate 的字段覆盖（非空字段）
            am.marker_title = Set(mp.marker_title);
            am.position = Set(mp.position);
            am.content = Set(mp.content);
            am.refresh_time = Set(mp.refresh_time);
            am.hidden_flag = Set(mp.hidden_flag);
            if mp.picture.is_some() {
                am.picture = Set(mp.picture);
            }
            if mp.video_path.is_some() {
                am.video_path = Set(mp.video_path);
            }
            let updated = marker_model::Entity::update_safety(am)?.exec(db).await?;
            // 删除 punctuate 记录
            mp_model::Entity::delete_by_id(mp.id).exec(db).await?;
            updated.id
        },
        MarkerPunctuateMethodType::Deleted => {
            // 删除：软删除 original_marker_id 对应的 marker
            let orig_id = mp
                .original_marker_id
                .ok_or_else(|| anyhow!("无法找到删除点位的原始id"))?;

            let old = marker_model::Entity::find_safety_by_id(orig_id)
                .one(db)
                .await?
                .ok_or_else(|| anyhow!("无法找到原始id对应的原始点位"))?;

            marker_model::Entity::delete_safety(old.into())?
                .exec(db)
                .await?;
            // 删除 punctuate 记录
            mp_model::Entity::delete_by_id(mp.id).exec(db).await?;
            orig_id
        },
    };

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "id": result_id
    }))))
}

/// 审核驳回：将 Reviewing 的打点状态改为 Rejected，附带审核备注。
///
/// 对应 Java `PunctuateAuditService.rejectPunctuate`。
pub async fn do_reject(
    _auth: AuthInfo,
    punctuate_id: i64,
    audit_remark: String,
) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let mp = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(punctuate_id))
        .filter(mp_model::Column::Status.eq(MarkerPunctuateStatus::Reviewing))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("无打点相关信息，或该打点不在审核中状态"))?;

    let mut am: mp_model::ActiveModel = mp.into();
    am.status = Set(MarkerPunctuateStatus::Rejected);
    am.audit_remark = Set(Some(audit_remark));
    mp_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 删除打点审核记录（软删除）
pub async fn do_delete(
    _auth: AuthInfo,
    punctuate_id: i64,
) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let mp = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(punctuate_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("打点信息不存在"))?;

    mp_model::Entity::delete_safety(mp.into())?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
