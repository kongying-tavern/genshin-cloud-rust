use anyhow::{Result, anyhow};

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect,
    prelude::*,
};

use _database::DB_CONN;
use _database::models::icon::icon_type as icon_type_model;
use _utils::models::common::EmptyResponse;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        IconTypeAddRequest, IconTypeUpdateRequest,
        icon_type::{IconTypeListRequest, IconTypeListResponse, IconTypeVO},
        wrapper::CommonResponse,
    },
};

// 更新图标类型
pub async fn do_update(
    auth: AuthInfo,
    payload: IconTypeUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    // 使用安全查找带乐观锁的函数
    let item = icon_type_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon type not found"))?;

    let mut am: icon_type_model::ActiveModel = item.into();
    // 更新基础字段
    am.name = Set(payload.base.name);
    am.parent_id = Set(payload.base.parent_id);
    am.is_final = Set(payload.base.is_final);
    // 版本由宏与 update_safety 处理

    icon_type_model::Entity::update_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 列表（分页 + 父级过滤，typeIdList 含 -1 时查根分类）
pub async fn do_list(
    _auth: AuthInfo,
    payload: IconTypeListRequest,
) -> Result<CommonResponse<IconTypeListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = icon_type_model::Entity::find_safety();
    if let Some(ids) = payload.type_id_list {
        let parents: Vec<i64> = ids.into_iter().filter(|&t| t > 0).collect();
        if parents.is_empty() {
            query = query.filter(icon_type_model::Column::ParentId.eq(-1));
        } else {
            query = query.filter(icon_type_model::Column::ParentId.is_in(parents));
        }
    }

    let total = query.clone().count(db).await? as i64;
    let size = payload.page.size.unwrap_or(10) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let items = query.limit(size).offset(offset).all(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(IconTypeVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: it.creator_id,
            updater_id: it.updater_id,
            del_flag: it.del_flag,

            name: it.name,
            parent_id: it.parent_id,
            is_final: it.is_final,
        });
    }
    Ok(CommonResponse::new(Ok(IconTypeListResponse {
        total,
        items: arr,
    })))
}

// 删除（软删除）
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let item = icon_type_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon type not found"))?;
    let mut am: icon_type_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    icon_type_model::Entity::delete_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 新增图标类型，返回新 ID
pub async fn do_add(auth: AuthInfo, payload: IconTypeAddRequest) -> Result<i64> {
    auth.require_non_anonymous()?;
    let now = chrono::Utc::now().naive_utc();

    let active = icon_type_model::ActiveModel {
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
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(res.id)
}
