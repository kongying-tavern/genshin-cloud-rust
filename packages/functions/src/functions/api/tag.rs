use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN,
    models::{icon::icon as icon_model, tag::tag as tag_model, tag::tag_type_link as ttl_model},
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        tag::{
            TagAddRequest, TagAddResponse, TagListRequest, TagListResponse, TagUpdateRequest,
            TagUpdateTypeRequest, TagVO,
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

/// 新增标签
pub async fn do_add(auth: AuthInfo, payload: TagAddRequest) -> Result<TagAddResponse> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let am = tag_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),
        tag: Set(payload.tag),
        icon_id: Set(payload.icon_id),
        hidden_flag: Set(Some(0)),
        sort_index: Set(Some(0)),
    };

    let res = tag_model::Entity::insert(am).exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(TagAddResponse {
        id: res.last_insert_id,
    })
}

/// 更新标签
pub async fn do_update(auth: AuthInfo, payload: TagUpdateRequest) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let t = t.ok_or(anyhow!("Tag not found"))?;
    let old_tag = t.tag.clone();
    let new_tag = payload.base.tag;
    let mut am: tag_model::ActiveModel = t.into();
    // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
    am.updater_id = Set(Some(auth.info.id));

    am.tag = Set(new_tag.clone());
    am.icon_id = Set(payload.base.icon_id);

    tag_model::Entity::update_safety(am)?.exec(db).await?;

    // 改名时同步 tag_type_link（该表以 tag_name 为键，否则旧关联悬空）
    if new_tag != old_tag {
        ttl_model::Entity::update_many()
            .col_expr(
                ttl_model::Column::TagName,
                sea_orm::sea_query::Expr::value(new_tag.clone()),
            )
            .filter(ttl_model::Column::TagName.eq(old_tag))
            .exec(db)
            .await?;
    }
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 标签列表（分页 + 模糊搜索）
pub async fn do_list(
    _auth: AuthInfo,
    payload: TagListRequest,
) -> Result<CommonResponse<TagListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = tag_model::Entity::find_safety();

    if let Some(tag) = payload.tag {
        query = query.filter(tag_model::Column::Tag.like(format!("%{}%", escape_like(&tag))));
    }
    if let Some(icon_id) = payload.icon_id {
        query = query.filter(tag_model::Column::IconId.eq(icon_id));
    }
    if let Some(tag_list) = payload.tag_list
        && !tag_list.is_empty()
    {
        query = query.filter(tag_model::Column::Tag.is_in(tag_list));
    }
    // typeIdList 有值时恒执行过滤：空命中集也必须过滤（返回空页），
    // 否则 `!tag_names.is_empty()` 守卫会丢弃过滤条件而返回全量数据。
    if let Some(type_id_list) = payload.type_id_list {
        let tag_names: Vec<String> = ttl_model::Entity::find_safety()
            .filter(ttl_model::Column::TypeId.is_in(type_id_list))
            .all(db)
            .await?
            .into_iter()
            .map(|l| l.tag_name)
            .collect();
        query = query.filter(tag_model::Column::Tag.is_in(tag_names));
    }

    let size = payload.page.size.unwrap_or(10).min(200) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await? as i64;
    let items = query.limit(size).offset(offset).all(db).await?;

    let list: Vec<TagVO> = items
        .into_iter()
        .map(|t| TagVO {
            version: t.version,
            id: t.id,
            create_time: Some(t.create_time.and_utc().timestamp_millis() as f64),
            update_time: t
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: t.creator_id,
            updater_id: t.updater_id,
            tag: t.tag,
            type_id_list: Vec::new(),
            icon_id: t.icon_id,
            url: String::new(),
        })
        .collect();

    Ok(CommonResponse::new(Ok(TagListResponse { total, list })))
}

/// 软删除标签
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety_by_id(id).one(db).await?;
    let t = t.ok_or(anyhow!("Tag not found"))?;
    let mut am: tag_model::ActiveModel = t.into();
    am.del_flag = Set(true);
    // 审计字段：软删也是修改，设置 update 组
    am.updater_id = Set(Some(auth.info.id));
    tag_model::Entity::delete_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
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

    // Java TagService.updateTypeInTag：typeIdList 中存在未创建的类型即报错
    if !payload.type_id_list.is_empty() {
        let existing: std::collections::HashSet<i64> =
            _database::models::tag::tag_type::Entity::find_safety()
                .filter(
                    _database::models::tag::tag_type::Column::Id
                        .is_in(payload.type_id_list.clone()),
                )
                .all(db)
                .await?
                .into_iter()
                .map(|t| t.id)
                .collect();
        if payload.type_id_list.iter().any(|t| !existing.contains(t)) {
            return Err(anyhow!("类型ID错误"));
        }
    }

    // 删除该 tag 的旧关联
    let links = ttl_model::Entity::find_safety()
        .filter(ttl_model::Column::TagName.eq(&payload.tag))
        .all(db)
        .await?;
    for mut link in links {
        // 审计字段：软删也是修改，设置 update 组（Model 原值随 into() 落库）
        link.updater_id = Some(auth.info.id);
        ttl_model::Entity::delete_safety(link.into())?
            .exec(db)
            .await?;
    }

    // 重建关联
    for type_id in payload.type_id_list {
        ttl_model::Entity::insert(ttl_model::ActiveModel {
            version: Set(1),
            id: NotSet,
            // 审计字段：新增时 create/update 两组全部设置
            create_time: Set(now),
            update_time: Set(Some(now)),
            creator_id: Set(Some(auth.info.id)),
            updater_id: Set(Some(auth.info.id)),
            del_flag: Set(false),
            type_id: Set(type_id),
            tag_name: Set(payload.tag.clone()),
        })
        .exec(db)
        .await?;
    }

    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 按标签名新增标签（前端 `createTag` 兼容路由，仅传标签名）：
/// 已存在同名标签时返回 `false`（对齐 Java 语义，前端据此回退为查询）。
pub async fn do_create_by_name(auth: AuthInfo, tag_name: String) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let exists = tag_model::Entity::find_safety()
        .filter(tag_model::Column::Tag.eq(&tag_name))
        .count(db)
        .await?;
    if exists > 0 {
        return Ok(CommonResponse::new(Ok(false)));
    }

    let now = Utc::now().naive_utc();
    tag_model::Entity::insert(tag_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),
        tag: Set(tag_name),
        icon_id: Set(0),
        hidden_flag: Set(Some(0)),
        sort_index: Set(Some(0)),
    })
    .exec(db)
    .await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 按标签名软删除标签（前端 `deleteTag` 兼容路由）。
pub async fn do_delete_by_name(auth: AuthInfo, tag_name: String) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety()
        .filter(tag_model::Column::Tag.eq(&tag_name))
        .one(db)
        .await?
        .ok_or(anyhow!("无删除的标签"))?;
    // Java deleteTag：先删 tag_type_link 再删 tag，避免悬空关联
    ttl_model::Entity::update_many()
        .col_expr(
            ttl_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(ttl_model::Column::TagName.eq(&tag_name))
        .exec(db)
        .await?;
    let mut am: tag_model::ActiveModel = t.into();
    am.del_flag = Set(true);
    // 审计字段：软删也是修改，设置 update 组
    am.updater_id = Set(Some(auth.info.id));
    tag_model::Entity::delete_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 按标签名更新图标绑定（前端 `updateTag` 兼容路由）。
pub async fn do_update_by_name(
    auth: AuthInfo,
    tag_name: String,
    icon_id: i64,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety()
        .filter(tag_model::Column::Tag.eq(&tag_name))
        .one(db)
        .await?
        .ok_or(anyhow!("Tag not found"))?;
    let mut am: tag_model::ActiveModel = t.into();
    // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
    am.updater_id = Set(Some(auth.info.id));
    am.icon_id = Set(icon_id);
    tag_model::Entity::update_safety(am)?.exec(db).await?;
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "IconTagBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 按标签名查询单个标签（前端 `getTag` 兼容路由），返回完整 TagVO。
pub async fn do_get_single(auth: AuthInfo, tag_name: String) -> Result<CommonResponse<TagVO>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let t = tag_model::Entity::find_safety()
        .filter(tag_model::Column::Tag.eq(&tag_name))
        .one(db)
        .await?
        .ok_or(anyhow!("Tag not found"))?;

    let type_id_list: Vec<i64> = ttl_model::Entity::find_safety()
        .filter(ttl_model::Column::TagName.eq(&tag_name))
        .all(db)
        .await?
        .into_iter()
        .map(|l| l.type_id)
        .collect();

    let url = icon_model::Entity::find_safety_by_id(t.icon_id)
        .one(db)
        .await?
        .map(|i| i.url)
        .unwrap_or_default();

    Ok(CommonResponse::new(Ok(TagVO {
        version: t.version,
        id: t.id,
        create_time: Some(t.create_time.and_utc().timestamp_millis() as f64),
        update_time: t
            .update_time
            .map(|dt| dt.and_utc().timestamp_millis() as f64),
        creator_id: t.creator_id,
        updater_id: t.updater_id,
        tag: t.tag,
        type_id_list,
        icon_id: t.icon_id,
        url,
    })))
}
