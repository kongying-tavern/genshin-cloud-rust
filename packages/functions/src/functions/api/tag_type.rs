use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{DB_CONN, models::tag::tag_type as tag_type_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        tag_type::{
            TagTypeBaseRequest, TagTypeListRequest, TagTypeListResponse, TagTypeUpdateRequest,
            TagTypeVO,
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

/// 新增标签类型
pub async fn do_add(auth: AuthInfo, payload: TagTypeBaseRequest) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let am = tag_type_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        name: Set(payload.name),
        parent_id: Set(payload.parent_id),
        is_final: Set(payload.is_final),
        sort_index: Set(Some(0)),
    };

    let res = tag_type_model::Entity::insert(am).exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(res.last_insert_id)))
}

/// 更新标签类型
pub async fn do_update(
    auth: AuthInfo,
    payload: TagTypeUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_type_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let t = t.ok_or(anyhow!("TagType not found"))?;
    let mut am: tag_type_model::ActiveModel = t.into();

    am.name = Set(payload.base.name);
    am.parent_id = Set(payload.base.parent_id);
    am.is_final = Set(payload.base.is_final);

    tag_type_model::Entity::update_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 标签类型列表（分页 + 模糊搜索 + 父级过滤）
pub async fn do_list(
    _auth: AuthInfo,
    payload: TagTypeListRequest,
) -> Result<CommonResponse<TagTypeListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = tag_type_model::Entity::find_safety();

    if let Some(name) = payload.name {
        query =
            query.filter(tag_type_model::Column::Name.like(format!("%{}%", escape_like(&name))));
    }
    if let Some(parent_id) = payload.parent_id {
        query = query.filter(tag_type_model::Column::ParentId.eq(parent_id));
    }
    // typeIdList 语义：[-1] 返回根类型（parent_id=-1），[nodeId] 返回其子级（parent_id IN）
    if let Some(type_list) = payload.type_id_list
        && !type_list.is_empty()
    {
        if type_list.contains(&-1) {
            query = query.filter(tag_type_model::Column::ParentId.eq(-1));
        } else {
            query = query.filter(tag_type_model::Column::ParentId.is_in(type_list));
        }
    }

    let size = payload.page.size.unwrap_or(10).min(200) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await? as i64;
    let items = query.limit(size).offset(offset).all(db).await?;

    let list: Vec<TagTypeVO> = items
        .into_iter()
        .map(|t| TagTypeVO {
            version: t.version,
            id: t.id,
            create_time: t.create_time.and_utc().timestamp_millis() as f64,
            update_time: t
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: t.creator_id,
            updater_id: t.updater_id,
            name: t.name,
            parent_id: t.parent_id,
            is_final: t.is_final,
        })
        .collect();

    Ok(CommonResponse::new(Ok(TagTypeListResponse { total, list })))
}

/// 软删除标签类型
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_type_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?;
    let t = t.ok_or(anyhow!("TagType not found"))?;
    let mut am: tag_type_model::ActiveModel = t.into();
    am.del_flag = Set(true);
    tag_type_model::Entity::delete_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
