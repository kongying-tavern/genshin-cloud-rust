use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN,
    models::{tag::tag as tag_model, tag::tag_type_link as ttl_model},
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        tag::{
            TagAddRequest, TagAddResponse, TagListRequest, TagListResponse, TagUpdateRequest,
            TagUpdateTypeRequest, TagVO,
        },
        wrapper::CommonResponse,
    },
};

/// 新增标签
pub async fn do_add(auth: AuthInfo, payload: TagAddRequest) -> Result<TagAddResponse> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let am = tag_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        tag: Set(payload.tag),
        icon_id: Set(payload.icon_id),
    };

    let res = tag_model::Entity::insert(am).exec(db).await?;
    Ok(TagAddResponse {
        id: res.last_insert_id,
    })
}

/// 更新标签
pub async fn do_update(
    auth: AuthInfo,
    payload: TagUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let t = t.ok_or(anyhow!("Tag not found"))?;
    let mut am: tag_model::ActiveModel = t.into();

    am.tag = Set(payload.base.tag);
    am.icon_id = Set(payload.base.icon_id);

    tag_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 标签列表（分页 + 模糊搜索）
pub async fn do_list(
    _auth: AuthInfo,
    payload: TagListRequest,
) -> Result<CommonResponse<TagListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = tag_model::Entity::find_safety();

    if let Some(tag) = payload.tag {
        query = query.filter(tag_model::Column::Tag.like(format!("%{}%", tag)));
    }
    if let Some(icon_id) = payload.icon_id {
        query = query.filter(tag_model::Column::IconId.eq(icon_id));
    }

    let size = payload.page.size.unwrap_or(10) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await? as i64;
    let items = query.limit(size).offset(offset).all(db).await?;

    let list: Vec<TagVO> = items
        .into_iter()
        .map(|t| TagVO {
            id: t.id,
            tag: t.tag,
            icon_id: t.icon_id,
        })
        .collect();

    Ok(CommonResponse::new(Ok(TagListResponse { total, list })))
}

/// 软删除标签
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety_by_id(id).one(db).await?;
    let t = t.ok_or(anyhow!("Tag not found"))?;
    let mut am: tag_model::ActiveModel = t.into();
    am.del_flag = Set(true);
    tag_model::Entity::delete_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 修改标签的分类信息（Java `updateTypeInTag`，仅供后台使用）：
/// 重建 `tag_type_link`（按 tag_name 全量替换 typeIdList）。
pub async fn do_update_type(
    auth: AuthInfo,
    payload: TagUpdateTypeRequest,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let _tag = tag_model::Entity::find_safety()
        .filter(tag_model::Column::Tag.eq(&payload.tag))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Tag not found"))?;

    // 删除该 tag 的旧关联
    let links = ttl_model::Entity::find_safety()
        .filter(ttl_model::Column::TagName.eq(&payload.tag))
        .all(db)
        .await?;
    for link in links {
        ttl_model::Entity::delete_safety(link.into())?
            .exec(db)
            .await?;
    }

    // 重建关联
    for type_id in payload.type_id_list {
        ttl_model::Entity::insert(ttl_model::ActiveModel {
            version: Set(0),
            id: NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),
            type_id: Set(type_id),
            tag_name: Set(payload.tag.clone()),
        })
        .exec(db)
        .await?;
    }

    Ok(CommonResponse::new(Ok(true)))
}
