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
            TagTypeAddResponse, TagTypeBaseRequest, TagTypeListRequest, TagTypeListResponse,
            TagTypeUpdateRequest, TagTypeVO,
        },
        wrapper::CommonResponse,
    },
};

/// 新增标签类型
pub async fn do_add(auth: AuthInfo, payload: TagTypeBaseRequest) -> Result<TagTypeAddResponse> {
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
    Ok(TagTypeAddResponse {
        id: res.last_insert_id,
    })
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
        query = query.filter(tag_type_model::Column::Name.like(format!("%{}%", name)));
    }
    if let Some(parent_id) = payload.parent_id {
        query = query.filter(tag_type_model::Column::ParentId.eq(parent_id));
    }

    let size = payload.page.size.unwrap_or(10) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await? as i64;
    let items = query.limit(size).offset(offset).all(db).await?;

    let list: Vec<TagTypeVO> = items
        .into_iter()
        .map(|t| TagTypeVO {
            id: t.id,
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
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
