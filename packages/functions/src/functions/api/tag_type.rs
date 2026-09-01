use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN,
    models::tag::{tag_type as tag_type_model, tag_type_link as ttl_model},
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
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
        sort_index: Set(Some(0)),
    };

    let res = tag_type_model::Entity::insert(am).exec(db).await?;
    // 父级不再是末端（Java updateTagTypeIsFinal）
    set_parent_is_final(db, payload.parent_id, false).await;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(res.last_insert_id)))
}

/// 更新标签类型（Java updateTagType：自环校验 + 父级 isFinal 联动 + 自身重算）
pub async fn do_update(
    auth: AuthInfo,
    payload: TagTypeUpdateRequest,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    if payload.id == payload.base.parent_id {
        return Err(anyhow!("标签类型ID不允许与父ID相同，会造成自身父子"));
    }

    let Some(t) = tag_type_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?
    else {
        return Ok(CommonResponse::new(Ok(false)));
    };
    // 父级变化时联动新旧父级的末端标志
    if t.parent_id != payload.base.parent_id {
        set_parent_is_final(db, payload.base.parent_id, false).await;
        recalc_parent_is_final(db, t.parent_id, true).await;
    }
    let mut am: tag_type_model::ActiveModel = t.into();

    am.name = Set(payload.base.name);
    am.parent_id = Set(payload.base.parent_id);
    // Java updateTagTypeIsFinal(tagType)：无子级才是末端
    let children = tag_type_model::Entity::find_safety()
        .filter(tag_type_model::Column::ParentId.eq(payload.id))
        .count(db)
        .await?;
    am.is_final = Set(children == 0);

    tag_type_model::Entity::update_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
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
    // typeIdList 语义（Java listTagType）：null → 根分类（parent_id=-1，不回退
    // 全量）；[-1] → 根分类；[nodeId] → 其子级（parent_id IN）
    match payload.type_id_list {
        None => {
            query = query.filter(tag_type_model::Column::ParentId.eq(-1));
        },
        Some(type_list) if !type_list.is_empty() => {
            if type_list.contains(&-1) {
                query = query.filter(tag_type_model::Column::ParentId.eq(-1));
            } else {
                query = query.filter(tag_type_model::Column::ParentId.is_in(type_list));
            }
        },
        Some(_) => {
            query = query.filter(tag_type_model::Column::ParentId.eq(-1));
        },
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

    Ok(CommonResponse::new(Ok(TagTypeListResponse {
        total,
        list,
        size: size as i64,
    })))
}

/// 删除标签类型（Java deleteTagType：递归删除整棵子树 + 标签类型关联 +
/// 父级 isFinal 重算）
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let Some(t) = tag_type_model::Entity::find_safety_by_id(id)
        .one(db)
        .await?
    else {
        return Ok(CommonResponse::new(Ok(false)));
    };
    let parent_id = t.parent_id;

    let mut now: Vec<i64> = vec![id];
    while !now.is_empty() {
        for chunk in now.chunks(1000) {
            // 标签类型关联（tag → type）
            for l in ttl_model::Entity::find()
                .filter(ttl_model::Column::TypeId.is_in(chunk))
                .all(db)
                .await?
            {
                let mut am: ttl_model::ActiveModel = l.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                ttl_model::Entity::update_safety(am)?.exec(db).await?;
            }
            // 类型本体
            for tt in tag_type_model::Entity::find_safety()
                .filter(tag_type_model::Column::Id.is_in(chunk))
                .all(db)
                .await?
            {
                let mut am: tag_type_model::ActiveModel = tt.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                tag_type_model::Entity::delete_safety(am)?.exec(db).await?;
            }
        }
        let mut children: Vec<i64> = Vec::new();
        for chunk in now.chunks(1000) {
            children.extend(
                tag_type_model::Entity::find_safety()
                    .filter(tag_type_model::Column::ParentId.is_in(chunk))
                    .select_only()
                    .column(tag_type_model::Column::Id)
                    .into_tuple::<i64>()
                    .all(db)
                    .await?,
            );
        }
        now = children;
    }
    recalc_parent_is_final(db, parent_id, false).await;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 父级存在（id > 0）时直接设置 isFinal（Java updateTagTypeIsFinal）。
async fn set_parent_is_final(db: &sea_orm::DatabaseConnection, parent_id: i64, is_final: bool) {
    if parent_id <= 0 {
        return;
    }
    let _: Result<()> = async {
        let Some(mut am): Option<tag_type_model::ActiveModel> =
            tag_type_model::Entity::find_safety_by_id(parent_id)
                .one(db)
                .await?
                .map(|m| m.into())
        else {
            return Ok(());
        };
        am.is_final = Set(is_final);
        tag_type_model::Entity::update_safety(am)?.exec(db).await?;
        Ok(())
    }
    .await;
}

/// 父级 isFinal 重算（Java recalculateTagTypeIsFinal）。
async fn recalc_parent_is_final(
    db: &sea_orm::DatabaseConnection,
    parent_id: i64,
    before_modify: bool,
) {
    if parent_id == 0 {
        return;
    }
    let count = tag_type_model::Entity::find_safety()
        .filter(tag_type_model::Column::ParentId.eq(parent_id))
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
