use anyhow::{Result, anyhow};

use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, QueryFilter, QuerySelect,
    prelude::*,
};

use _database::DB_CONN;
use _database::models::icon::{icon_type as icon_type_model, icon_type_link as itl_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        IconTypeAddRequest, IconTypeUpdateRequest,
        icon_type::{IconTypeListRequest, IconTypeListResponse, IconTypeVO},
        wrapper::CommonResponse,
    },
};

/// Java `IconTypeService` 同文案：禁止自身父子。
fn check_id_parent(id: i64, parent_id: i64) -> Result<()> {
    if id == parent_id {
        return Err(anyhow!("图标类型ID不允许与父ID相同，会造成自身父子"));
    }
    Ok(())
}

/// 父级存在（id > 0）时直接设置 isFinal（Java updateIconTypeIsFinal）。
async fn set_parent_is_final(db: &sea_orm::DatabaseConnection, parent_id: i64, is_final: bool) {
    if parent_id <= 0 {
        return;
    }
    let _: Result<()> = async {
        let Some(mut am): Option<icon_type_model::ActiveModel> =
            icon_type_model::Entity::find_safety_by_id(parent_id)
                .one(db)
                .await?
                .map(|m| m.into())
        else {
            return Ok(());
        };
        am.is_final = Set(is_final);
        icon_type_model::Entity::update_safety(am)?.exec(db).await?;
        Ok(())
    }
    .await;
}

/// 父级 isFinal 重算（Java recalculateIconTypeIsFinal）：
/// 剩余子级数 == (before_modify ? 1 : 0) 时置 true，否则 false。
async fn recalc_parent_is_final(
    db: &sea_orm::DatabaseConnection,
    parent_id: i64,
    before_modify: bool,
) {
    if parent_id == 0 {
        return;
    }
    let count = icon_type_model::Entity::find_safety()
        .filter(icon_type_model::Column::ParentId.eq(parent_id))
        .count(db)
        .await
        .unwrap_or(0);
    let target = if before_modify {
        count == 1
    } else {
        count == 0
    };
    set_parent_is_final(db, parent_id, target).await;
}

// 更新图标类型（Java updateIconType：自环校验 + 父级 isFinal 联动 + 自身重算）
pub async fn do_update(
    auth: AuthInfo,
    payload: IconTypeUpdateRequest,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    check_id_parent(payload.id, payload.base.parent_id)?;

    let Some(item) = icon_type_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?
    else {
        // Java：实体不存在返回 false（HTTP 200 + R{data:false}）
        return Ok(CommonResponse::new(Ok(false)));
    };

    // 父级变化时联动新旧父级的末端标志
    if item.parent_id != payload.base.parent_id {
        set_parent_is_final(db, payload.base.parent_id, false).await;
        recalc_parent_is_final(db, item.parent_id, true).await;
    }

    let mut am: icon_type_model::ActiveModel = item.into();
    am.name = Set(payload.base.name);
    am.parent_id = Set(payload.base.parent_id);
    // Java updateIconTypeIsFinal(iconType)：请求中的 isFinal 会被重算覆盖 ——
    // 无子级才是末端
    let children = icon_type_model::Entity::find_safety()
        .filter(icon_type_model::Column::ParentId.eq(payload.id))
        .count(db)
        .await?;
    am.is_final = Set(children == 0);
    icon_type_model::Entity::update_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(true)))
}

// 列表（Java listIconType：typeIdList 为 null 时默认查根分类 parent IN (-1)）
pub async fn do_list(
    _auth: AuthInfo,
    payload: IconTypeListRequest,
) -> Result<CommonResponse<IconTypeListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = icon_type_model::Entity::find_safety();
    match payload.type_id_list {
        // Java：null → Collections.singletonList(-1L)（根分类）；不回退全量
        None => {
            query = query.filter(icon_type_model::Column::ParentId.eq(-1));
        },
        Some(ids) => {
            let parents: Vec<i64> = ids.into_iter().filter(|&t| t > 0).collect();
            if parents.is_empty() {
                query = query.filter(icon_type_model::Column::ParentId.eq(-1));
            } else {
                query = query.filter(icon_type_model::Column::ParentId.is_in(parents));
            }
        },
    }

    let total = query.clone().count(db).await? as i64;
    let size = payload.page.size.unwrap_or(10).min(200) as u64;
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
        size: size as i64,
        items: arr,
    })))
}

// 删除（Java deleteIconType：递归删除整棵子树 + 图标类型关联 + 父级 isFinal 重算）
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let Some(item) = icon_type_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
    else {
        return Ok(CommonResponse::new(Ok(false)));
    };
    let parent_id = item.parent_id;

    // BFS 逐层：软删类型 → 删除 icon_type_link → 收集子级
    let mut now: Vec<i64> = vec![id];
    while !now.is_empty() {
        for chunk in now.chunks(1000) {
            // 类型关联（icon → type）
            for l in itl_model::Entity::find()
                .filter(itl_model::Column::TypeId.is_in(chunk))
                .all(db)
                .await?
            {
                let mut am: itl_model::ActiveModel = l.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                itl_model::Entity::update_safety(am)?.exec(db).await?;
            }
            // 类型本体
            for t in icon_type_model::Entity::find_safety()
                .filter(icon_type_model::Column::Id.is_in(chunk))
                .all(db)
                .await?
            {
                let mut am: icon_type_model::ActiveModel = t.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                icon_type_model::Entity::delete_safety(am)?.exec(db).await?;
            }
        }
        let mut children: Vec<i64> = Vec::new();
        for chunk in now.chunks(1000) {
            children.extend(
                icon_type_model::Entity::find_safety()
                    .filter(icon_type_model::Column::ParentId.is_in(chunk))
                    .select_only()
                    .column(icon_type_model::Column::Id)
                    .into_tuple::<i64>()
                    .all(db)
                    .await?,
            );
        }
        now = children;
    }
    recalc_parent_is_final(db, parent_id, false).await;
    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(true)))
}

// 新增图标类型（Java addIconType：isFinal 恒 true + 父级置非末端）
pub async fn do_add(auth: AuthInfo, payload: IconTypeAddRequest) -> Result<i64> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let now = chrono::Utc::now().naive_utc();

    let active = icon_type_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),

        name: Set(payload.name),
        parent_id: Set(payload.parent_id),
        // Java withIsFinal(true)：新增节点无子级，必为末端
        is_final: Set(true),
    };

    let res = active.insert(db).await?;
    // 父级不再是末端
    set_parent_is_final(db, payload.parent_id, false).await;
    super::binary_doc::invalidate_doc_cache().await;
    Ok(res.id)
}
