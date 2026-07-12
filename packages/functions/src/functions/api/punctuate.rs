use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::marker::marker_punctuate as mp_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        punctuate::PunctuateData,
        wrapper::{CommonResponse, Pagination},
    },
    types::MarkerPunctuateStatus,
};

/// 暂存 / 提交打点
///
/// - `status == Pending`：暂存（STAGE），用户可在后续提交审核
/// - `status == Reviewing`：提交审核（COMMIT），从 Pending 或 Rejected 晋升
///
/// 对应 Java `PunctuateService.stage` / `PunctuateService.commit`。
pub async fn do_submit(
    _auth: AuthInfo,
    payload: PunctuateData,
) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    match payload.status {
        MarkerPunctuateStatus::Pending => {
            // STAGE: 新建或覆盖暂存（按 punctuate_id + STAGE|REJECTED 查找并替换）
            let existing = mp_model::Entity::find_safety()
                .filter(mp_model::Column::PunctuateId.eq(payload.punctuate_id as i64))
                .filter(mp_model::Column::Status.is_in([
                    MarkerPunctuateStatus::Pending,
                    MarkerPunctuateStatus::Rejected,
                ]))
                .one(db)
                .await?;

            if let Some(m) = existing {
                // 更新已有的暂存记录
                let mut am: mp_model::ActiveModel = m.into();
                apply_punctuate_fields(&mut am, &payload);
                mp_model::Entity::update_safety(am)?.exec(db).await?;
            } else {
                // 新建暂存记录
                let am = new_punctuate_active_model(&payload, now);
                mp_model::Entity::insert(am).exec(db).await?;
            }
        },
        MarkerPunctuateStatus::Reviewing => {
            // COMMIT: 将 STAGE 或 REJECTED 的记录状态改为 Reviewing
            let m = mp_model::Entity::find_safety()
                .filter(mp_model::Column::PunctuateId.eq(payload.punctuate_id as i64))
                .filter(mp_model::Column::Status.is_in([
                    MarkerPunctuateStatus::Pending,
                    MarkerPunctuateStatus::Rejected,
                ]))
                .one(db)
                .await?
                .ok_or_else(|| anyhow!("无待提交的打点信息"))?;

            let mut am: mp_model::ActiveModel = m.into();
            am.status = Set(MarkerPunctuateStatus::Reviewing);
            apply_punctuate_fields(&mut am, &payload);
            mp_model::Entity::update_safety(am)?.exec(db).await?;
        },
        MarkerPunctuateStatus::Rejected => {
            return Err(anyhow!("不能直接将状态设为不通过；需通过审核驳回流程"));
        },
    }

    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 更新打点内容（仅 Pending/Rejected 状态可改）
pub async fn do_update(
    _auth: AuthInfo,
    payload: PunctuateData,
) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let m = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(payload.punctuate_id as i64))
        .filter(mp_model::Column::Status.is_in([
            MarkerPunctuateStatus::Pending,
            MarkerPunctuateStatus::Rejected,
        ]))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("打点信息不存在或已提交，无法修改"))?;

    let mut am: mp_model::ActiveModel = m.into();
    apply_punctuate_fields(&mut am, &payload);
    mp_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 分页查询所有打点信息
pub async fn do_get_page(
    _auth: AuthInfo,
    payload: Pagination,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let query = mp_model::Entity::find_safety();
    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "list": items,
    }))))
}

/// 按提交者分页查询打点信息
pub async fn do_get_page_by_author(
    _auth: AuthInfo,
    author_id: i64,
    payload: Pagination,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &DB_CONN.wait().pg_conn;
    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let query = mp_model::Entity::find_safety().filter(mp_model::Column::Author.eq(author_id));
    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    Ok(CommonResponse::new(Ok(serde_json::json!({
        "total": total,
        "list": items,
    }))))
}

/// 删除打点记录（软删除）
pub async fn do_delete(
    _auth: AuthInfo,
    punctuate_id: i64,
) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let m = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(punctuate_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("打点信息不存在"))?;

    mp_model::Entity::delete_safety(m.into())?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// 将 PunctuateData 的字段写入 ActiveModel（不改 status / id / version）。
fn apply_punctuate_fields(am: &mut mp_model::ActiveModel, p: &PunctuateData) {
    am.marker_title = Set(Some(p.marker_title.clone()));
    am.content = Set(p.content.clone().unwrap_or_default());
    am.position = Set(p.position.clone());
    am.picture = Set(p.picture.clone());
    am.video_path = Set(p.video_path.clone());
    am.method_type = Set(p.method_type);
    am.hidden_flag = Set(p.hidden_flag);
    am.original_marker_id = Set(p.original_marker_id.map(|f| f as i64));
    am.refresh_time = Set(p.refresh_time.unwrap_or(0));
    am.item_list =
        Set(serde_json::to_value(&p.item_list).unwrap_or(serde_json::Value::Array(Vec::new())));
    am.extra = Set(p
        .extra
        .as_ref()
        .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null)));
}

/// 构造一条新的 marker_punctuate ActiveModel（状态 = payload.status）。
fn new_punctuate_active_model(
    p: &PunctuateData,
    now: chrono::NaiveDateTime,
) -> mp_model::ActiveModel {
    mp_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(p.author)),
        updater_id: Set(None),
        del_flag: Set(false),
        punctuate_id: Set(p.punctuate_id as i64),
        original_marker_id: Set(p.original_marker_id.map(|f| f as i64)),
        marker_title: Set(Some(p.marker_title.clone())),
        item_list: Set(
            serde_json::to_value(&p.item_list).unwrap_or(serde_json::Value::Array(Vec::new()))
        ),
        position: Set(p.position.clone()),
        content: Set(p.content.clone().unwrap_or_default()),
        picture: Set(p.picture.clone()),
        marker_creator_id: Set(p.marker_creator_id),
        picture_creator_id: Set(p.picture_creator_id),
        video_path: Set(p.video_path.clone()),
        author: Set(p.author),
        status: Set(p.status),
        audit_remark: Set(None),
        method_type: Set(p.method_type),
        refresh_time: Set(p.refresh_time.unwrap_or(0)),
        hidden_flag: Set(p.hidden_flag),
        extra: Set(p
            .extra
            .as_ref()
            .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null))),
    }
}
