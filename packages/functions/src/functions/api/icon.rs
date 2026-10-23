use anyhow::Result;

use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QuerySelect,
    prelude::*,
};

use _database::DatabaseConnectionMap;
use _database::models::icon::icon as icon_model;
use _database::models::icon::icon_type_link as icon_type_link_model;
use _utils::{
    db_operations::SafeEntityTrait,
    errors::DomainError,
    jwt::AuthInfo,
    models::{
        IconAddRequest, IconListRequest, IconUpdateRequest,
        icon::{IconListResponse, IconVO},
        wrapper::CommonResponse,
    },
    text::escape_like,
};

// 新增图标
pub async fn do_add(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    payload: IconAddRequest,
) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();

    // Java createIcon：标签不能为空，且同标签不可重复（唯一性在业务层，
    // 库表无唯一索引——与线上 Java 行为一致）。
    let tag = payload.tag.trim().to_string();
    if tag.is_empty() {
        return Err(DomainError::Business("图标标签不能为空".into()).into());
    }
    if icon_model::Entity::find_safety()
        .filter(icon_model::Column::Tag.eq(tag.clone()))
        .one(&db.pg_conn)
        .await?
        .is_some()
    {
        return Err(DomainError::Business(format!("图标标签 [{tag}]已存在")).into());
    }

    let active = icon_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),

        tag: Set(tag),
        description: Set(payload.description.unwrap_or_default()),
        url: Set(payload.url),
        url_variants: Set(payload.url_variants),
    };

    let res = active.insert(&db.pg_conn).await?;
    // 类型关联（Java createIcon）：typeIdList 非空时校验类型存在并写入
    // icon_type_link；类型 ID 不存在报「类型ID错误」。
    if let Some(type_ids) = payload.type_id_list.filter(|l| !l.is_empty()) {
        write_icon_type_links(&db.pg_conn, auth.info.id, res.id, &type_ids).await?;
    }
    super::binary_doc::invalidate_doc_cache(db).await;
    Ok(CommonResponse::new(Ok(res.id)))
}

/// 校验类型 ID 全部存在（对齐 Java「类型ID错误」文案）并重建该图标的
/// icon_type_link（Java updateIcon 的 diff-then-replace 语义：变更时全删重建）。
async fn write_icon_type_links(
    db: &sea_orm::DatabaseConnection,
    operator_id: i64,
    icon_id: i64,
    type_ids: &[i64],
) -> Result<()> {
    use _database::models::icon::icon_type as icon_type_model;
    let existing: Vec<i64> = icon_type_model::Entity::find_safety()
        .filter(icon_type_model::Column::Id.is_in(type_ids.to_vec()))
        .select_only()
        .column(icon_type_model::Column::Id)
        .into_tuple::<i64>()
        .all(db)
        .await?;
    if existing.len() != type_ids.len() {
        return Err(DomainError::Business("类型ID错误".into()).into());
    }
    // replace semantics: clear then insert
    icon_type_link_model::Entity::delete_many()
        .filter(icon_type_link_model::Column::IconId.eq(icon_id))
        .exec(db)
        .await?;
    let now = chrono::Utc::now().naive_utc();
    for tid in type_ids {
        icon_type_link_model::ActiveModel {
            version: Set(1),
            id: NotSet,
            // 审计字段：新增时 create/update 两组全部设置
            create_time: Set(now),
            update_time: Set(Some(now)),
            creator_id: Set(Some(operator_id)),
            updater_id: Set(Some(operator_id)),
            del_flag: Set(false),
            icon_id: Set(icon_id),
            type_id: Set(*tid),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

// 列表查询（支持分页）
pub async fn do_list(
    db: &DatabaseConnectionMap,
    _auth: AuthInfo,
    payload: IconListRequest,
) -> Result<CommonResponse<IconListResponse>> {
    let mut query = icon_model::Entity::find_safety();
    if let Some(creator) = payload.creator {
        query = query.filter(icon_model::Column::CreatorId.eq(creator));
    }
    // iconList（旧字段）与 iconIdList（前端契约）均生效
    let id_list = payload.icon_list.or(payload.icon_id_list);
    if let Some(ids) = id_list
        && !ids.is_empty()
    {
        query = query.filter(icon_model::Column::Id.is_in(ids));
    }
    if let Some(tag) = payload.tag {
        query = query.filter(icon_model::Column::Tag.like(format!("%{}%", escape_like(&tag))));
    }
    // 按图标分类（icon_type_link）过滤：icon_id IN (SELECT icon_id FROM icon_type_link WHERE type_id IN ...)
    // typeIdList 有值时恒执行过滤：空命中集也必须过滤（返回空页），
    // 否则 `!icon_ids.is_empty()` 守卫会丢弃过滤条件而返回全量数据。
    if let Some(type_ids) = payload.type_id_list {
        let icon_ids: Vec<i64> = icon_type_link_model::Entity::find_safety()
            .filter(icon_type_link_model::Column::TypeId.is_in(type_ids))
            .all(&db.pg_conn)
            .await?
            .into_iter()
            .map(|l| l.icon_id)
            .collect();
        query = query.filter(icon_model::Column::Id.is_in(icon_ids));
    }

    let total = query.clone().count(&db.pg_conn).await?;

    let mut select = query;
    if let Some(current) = payload.page.current
        && let Some(size) = payload.page.size
    {
        let size = size.min(200);
        let offset = (current.saturating_sub(1) as u64).saturating_mul(size as u64);
        select = select.limit(size as u64).offset(offset);
    }

    let items = select.all(&db.pg_conn).await?;
    let type_map = icon_type_map(&db.pg_conn).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in &items {
        arr.push(icon_to_vo(it, &type_map));
    }
    // 前端图标管理页读取信封 users 显示创建者昵称（与 marker/history 一致）。
    let creator_ids: std::collections::HashSet<i64> =
        items.iter().filter_map(|it| it.creator_id).collect();
    let users = super::sys_user_map(&db.pg_conn, &creator_ids).await?;
    let size = payload.page.size.unwrap_or(10).min(200);
    let resp = IconListResponse {
        total: total as i64,
        size: size as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(resp)).with_users(users))
}

// 获取单个图标（前端契约 RIconVo：data 直接是 IconVO）
pub async fn do_get_single(
    db: &DatabaseConnectionMap,
    _auth: AuthInfo,
    id: i64,
) -> Result<CommonResponse<IconVO>> {
    let item = icon_model::Entity::find_safety_by_id(id)
        .one(&db.pg_conn)
        .await?;
    let item = item.ok_or_else(|| DomainError::Business("Icon not found".into()))?;
    let type_map = icon_type_map(&db.pg_conn).await?;
    Ok(CommonResponse::new(Ok(icon_to_vo(&item, &type_map))))
}

// 删除（软删除）
pub async fn do_delete(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    id: i64,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let Some(item) = icon_model::Entity::find_safety_by_id(id)
        .one(&db.pg_conn)
        .await?
    else {
        return Ok(CommonResponse::new(Ok(false)));
    };
    let mut am: icon_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    // 审计字段：软删也是修改，设置 update 组
    am.updater_id = Set(Some(auth.info.id));
    icon_model::Entity::delete_safety(am)?
        .exec(&db.pg_conn)
        .await?;
    // Java deleteIcon：同步删除 icon_type_link，避免悬空关联
    _database::models::icon::icon_type_link::Entity::update_many()
        .col_expr(
            _database::models::icon::icon_type_link::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(_database::models::icon::icon_type_link::Column::IconId.eq(id))
        .exec(&db.pg_conn)
        .await?;
    super::binary_doc::invalidate_doc_cache(db).await;
    Ok(CommonResponse::new(Ok(true)))
}

// 更新图标
pub async fn do_update(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    payload: IconUpdateRequest,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let item = icon_model::Entity::find_safety_by_id(payload.id)
        .one(&db.pg_conn)
        .await?;
    let item = item.ok_or_else(|| DomainError::Business("Icon not found".into()))?;
    let old_url_variants = item.url_variants.clone();
    let mut am: icon_model::ActiveModel = item.into();
    // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
    am.updater_id = Set(Some(auth.info.id));
    am.tag = Set(payload.base.tag.clone());
    am.url = Set(payload.base.url.clone());
    am.description = Set(payload.base.description.clone().unwrap_or_default());
    // Java JsonUtils.merge：urlVariants 按 map 合并——新值为 null 的键删除，
    // 其余覆盖；未提交该字段则保持原值（不是整体替换）。
    am.url_variants = Set(merge_url_variants(
        old_url_variants.as_ref(),
        payload.base.url_variants.as_ref(),
    ));
    icon_model::Entity::update_safety(am)?
        .exec(&db.pg_conn)
        .await?;
    // 类型关联（Java updateIcon）：typeIdList 缺省不改动（避免误清空），
    // 提供时按 replace 语义重建。
    if let Some(type_ids) = payload.base.type_id_list.clone() {
        write_icon_type_links(&db.pg_conn, auth.info.id, payload.id, &type_ids).await?;
    }
    super::binary_doc::invalidate_doc_cache(db).await;
    Ok(CommonResponse::new(Ok(true)))
}

/// icon_id -> [type_id] 反查映射（IconVO.typeIdList 数据源）。
pub(crate) async fn icon_type_map(
    db: &sea_orm::DatabaseConnection,
) -> Result<std::collections::HashMap<i64, Vec<i64>>> {
    let mut map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for l in icon_type_link_model::Entity::find_safety().all(db).await? {
        map.entry(l.icon_id).or_default().push(l.type_id);
    }
    Ok(map)
}

/// 实体行 → 完整 `IconVO`（Java `IconVo` 全字段契约）。
fn icon_to_vo(
    it: &icon_model::Model,
    type_map: &std::collections::HashMap<i64, Vec<i64>>,
) -> IconVO {
    IconVO {
        id: it.id,
        version: it.version,
        creator_id: it.creator_id,
        create_time: it.create_time.and_utc().timestamp_millis() as f64,
        updater_id: it.updater_id,
        update_time: it
            .update_time
            .map(|dt| dt.and_utc().timestamp_millis() as f64),
        tag: it.tag.clone(),
        type_id_list: type_map.get(&it.id).cloned().unwrap_or_default(),
        url: super::rewrite_image_url(&it.url),
        url_variants: it.url_variants.clone(),
        description: it.description.clone(),
    }
}

/// icon_id -> tag.tag 映射（前端 sprite 的 tagCoordMap key 是 tag 表的
/// `tag` 字段（如 "陨石碎片"），不是 icon 表的 tag 描述）。
pub(crate) async fn icon_tag_map(
    db: &sea_orm::DatabaseConnection,
) -> Result<std::collections::HashMap<i64, String>> {
    use _database::models::tag::tag as tag_model;
    let mut map = std::collections::HashMap::new();
    for t in tag_model::Entity::find_safety().all(db).await? {
        map.entry(t.icon_id).or_insert(t.tag);
    }
    Ok(map)
}

/// tag 名 -> icon_id 映射（前端 iconTag → 图标 ID，取第一条匹配）。
pub(crate) async fn icon_id_by_tag(
    db: &sea_orm::DatabaseConnection,
    tag: &str,
) -> Result<Option<i64>> {
    use _database::models::tag::tag as tag_model;
    let t = tag_model::Entity::find_safety()
        .filter(tag_model::Column::Tag.eq(tag))
        .one(db)
        .await?;
    Ok(t.map(|t| t.icon_id))
}

/// `urlVariants` 的 Java `JsonUtils.merge` 语义：以旧 map 为底，逐键应用
/// 新 map（null 表示删除该键）；新值非对象时整体替换。
fn merge_url_variants(
    old: Option<&serde_json::Value>,
    new: Option<&serde_json::Value>,
) -> Option<serde_json::Value> {
    match new {
        None => old.cloned(),
        Some(serde_json::Value::Object(new_map)) => {
            let mut merged = old.and_then(|v| v.as_object().cloned()).unwrap_or_default();
            for (k, v) in new_map {
                if v.is_null() {
                    merged.remove(k);
                } else {
                    merged.insert(k.clone(), v.clone());
                }
            }
            Some(serde_json::Value::Object(merged))
        },
        Some(other) => Some(other.clone()),
    }
}
