use anyhow::{anyhow, Result};

use sea_orm::{prelude::*, ActiveValue::Set};

use _database::models::icon::icon_type as icon_type_model;
use _database::DB_CONN;
use _utils::models::common::EmptyResponse;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        icon_type::{IconTypeListResponse, IconTypeVO},
        wrapper::CommonResponse,
        IconTypeAddRequest, IconTypeUpdateRequest,
    },
};

// 更新图标类型
pub async fn do_update(
    _auth: AuthInfo,
    payload: IconTypeUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
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

    icon_type_model::Entity::update_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 列表（返回 JSON）
pub async fn do_list(_auth: AuthInfo) -> Result<CommonResponse<IconTypeListResponse>> {
    let items = icon_type_model::Entity::find_safety()
        .all(&DB_CONN.wait().pg_conn)
        .await?;
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
    Ok(CommonResponse::new(Ok(IconTypeListResponse { items: arr })))
}

// 删除（软删除）
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let item = icon_type_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon type not found"))?;
    let mut am: icon_type_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    icon_type_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 新增图标类型，返回新 ID
pub async fn do_add(_auth: AuthInfo, payload: IconTypeAddRequest) -> Result<i64> {
    let now = chrono::Utc::now().naive_utc();

    let active = icon_type_model::ActiveModel {
        version: Set(0),
        id: Set(0),
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
