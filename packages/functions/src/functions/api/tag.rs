use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::tag::tag as tag_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        tag::{
            TagAddRequest, TagAddResponse, TagListRequest, TagListResponse, TagUpdateRequest, TagVO,
        },
        wrapper::CommonResponse,
    },
};

/// 新增标签
pub async fn do_add(_auth: AuthInfo, payload: TagAddRequest) -> Result<TagAddResponse> {
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let am = tag_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        tag: Set(payload.tag),
        icon_id: Set(payload.icon_id),
        hidden_flag: Set(_utils::types::HiddenFlag::Visible),
        sort_index: Set(0),
    };

    let res = tag_model::Entity::insert(am).exec(db).await?;
    Ok(TagAddResponse {
        id: res.last_insert_id,
    })
}

/// 更新标签
pub async fn do_update(
    _auth: AuthInfo,
    payload: TagUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
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
            hidden_flag: t.hidden_flag,
            sort_index: t.sort_index,
        })
        .collect();

    Ok(CommonResponse::new(Ok(TagListResponse { total, list })))
}

/// 软删除标签
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety_by_id(id).one(db).await?;
    let t = t.ok_or(anyhow!("Tag not found"))?;
    let mut am: tag_model::ActiveModel = t.into();
    am.del_flag = Set(true);
    tag_model::Entity::delete_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
