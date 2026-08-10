use anyhow::{Result, anyhow};

use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QuerySelect,
    prelude::*,
};

use _database::DB_CONN;
use _database::models::icon::icon as icon_model;
use _database::models::icon::icon_type_link as icon_type_link_model;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        IconAddRequest, IconListRequest, IconUpdateRequest,
        icon::{IconListResponse, IconVO},
        wrapper::CommonResponse,
    },
};

// 新增图标
/// 转义 LIKE 通配符（% _ \），防止输入被当作模糊匹配通配符放大（PG 默认 ESCAPE 为反斜杠）。
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub async fn do_add(auth: AuthInfo, payload: IconAddRequest) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();

    let active = icon_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(None),
        del_flag: Set(false),

        tag: Set(payload.name),
        description: Set("".into()),
        url: Set(payload.url),
        url_variants: Set(None),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(res.id)))
}

// 列表查询（支持分页）
pub async fn do_list(
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
    if let Some(name) = payload.name {
        query = query.filter(icon_model::Column::Tag.like(format!("%{}%", escape_like(&name))));
    }
    // 按图标分类（icon_type_link）过滤：icon_id IN (SELECT icon_id FROM icon_type_link WHERE type_id IN ...)
    if let Some(type_ids) = payload.type_id_list {
        let icon_ids: Vec<i64> = icon_type_link_model::Entity::find_safety()
            .filter(icon_type_link_model::Column::TypeId.is_in(type_ids))
            .all(&DB_CONN.wait().pg_conn)
            .await?
            .into_iter()
            .map(|l| l.icon_id)
            .collect();
        if !icon_ids.is_empty() {
            query = query.filter(icon_model::Column::Id.is_in(icon_ids));
        }
    }

    let total = query.clone().count(&DB_CONN.wait().pg_conn).await?;

    let mut select = query;
    if let Some(current) = payload.page.current
        && let Some(size) = payload.page.size
    {
        let size = size.min(200);
        let offset = (current.saturating_sub(1) as u64).saturating_mul(size as u64);
        select = select.limit(size as u64).offset(offset);
    }

    let items = select.all(&DB_CONN.wait().pg_conn).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(IconVO {
            id: it.id,
            version: it.version,
            name: it.tag,
            url: it.url,
        });
    }
    let payload = IconListResponse {
        total: total as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

// 获取单个图标（前端契约 RIconVo：data 直接是 IconVO）
pub async fn do_get_single(_auth: AuthInfo, id: i64) -> Result<CommonResponse<IconVO>> {
    let item = icon_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    Ok(CommonResponse::new(Ok(IconVO {
        id: item.id,
        version: item.version,
        name: item.tag,
        url: item.url,
    })))
}

// 删除（软删除）
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let item = icon_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let mut am: icon_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    icon_model::Entity::delete_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(())))
}

// 更新图标
pub async fn do_update(auth: AuthInfo, payload: IconUpdateRequest) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let item = icon_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let mut am: icon_model::ActiveModel = item.into();
    am.tag = Set(payload.base.name);
    am.url = Set(payload.base.url);
    icon_model::Entity::update_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(())))
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
