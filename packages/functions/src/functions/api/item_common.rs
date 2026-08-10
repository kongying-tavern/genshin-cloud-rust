//! 公用物品（地区公用物品）业务层。
//!
//! 对齐 Java `ItemCommonService`：`item_area_public` 是 **item 的关联表**
//! （把现有 item 标记为公用，按名称去重），不是 item 表本身。

use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder, QuerySelect, prelude::*};

use _utils::{
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        item::{ItemAreaPublicListResponse, ItemAreaPublicVo},
        wrapper::CommonResponse,
        wrapper::Pagination,
    },
};

use _utils::db_operations::SafeEntityTrait;

use _database::{
    DB_CONN,
    models::{area::item_area_public as iap_model, item::item as item_model},
};

use super::item::{item_to_vo, marker_count_map, type_id_map};

/// 批量添加上限（防恶意大列表）。
const MAX_BATCH: usize = 1000;

/// `POST /item_common/get/list`
///
/// 列出公用物品：分页查询 `item_area_public` 关联表，组合 item 信息。
/// 响应为 `ItemAreaPublicVo`（ItemVO + itemId），与前端 ItemAreaPublicVo 契约一致。
pub async fn do_get_list(
    _auth: AuthInfo,
    payload: Pagination,
) -> Result<CommonResponse<ItemAreaPublicListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let size = payload.size.unwrap_or(10).min(200) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = iap_model::Entity::find_safety().clone().count(db).await?;
    let links = iap_model::Entity::find_safety()
        .order_by(iap_model::Column::Id, sea_orm::Order::Desc)
        .limit(size)
        .offset(offset)
        .all(db)
        .await?;

    let ids: Vec<i64> = links.iter().map(|l| l.item_id).collect();
    let mut by_id = std::collections::HashMap::with_capacity(ids.len());
    if !ids.is_empty() {
        for it in item_model::Entity::find_safety()
            .filter(item_model::Column::Id.is_in(ids))
            .all(db)
            .await?
        {
            by_id.insert(it.id, it);
        }
    }

    let mut arr = Vec::with_capacity(links.len());
    let type_map = type_id_map(db).await?;
    let icon_tag_map = super::icon::icon_tag_map(db).await?;
    let count_map = marker_count_map(db).await?;
    for link in links {
        if let Some(it) = by_id.get(&link.item_id) {
            arr.push(ItemAreaPublicVo {
                item_id: it.id,
                item: item_to_vo(it, &type_map, &icon_tag_map, &count_map),
            });
        }
    }

    let payload = ItemAreaPublicListResponse {
        total: total as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

/// `PUT /item_common/add`
///
/// 对齐 Java：把 itemId 列表中**名称尚未成为公用物品**的 item 批量标记为
/// 公用（写入 `item_area_public`）。同名 item 只取第一个；名称已存在于
/// 关联表中的跳过。返回是否成功（至少插入一条）。
pub async fn do_add(auth: AuthInfo, item_id_list: Vec<i64>) -> Result<CommonResponse<bool>> {
    if item_id_list.len() > MAX_BATCH {
        return Err(anyhow!(
            "batch too large: {} > {}",
            item_id_list.len(),
            MAX_BATCH
        ));
    }
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    // 已存在的公用名称集合（Java：过滤掉名称已存在的）。
    let existing_links = iap_model::Entity::find_safety().all(db).await?;
    let existing_ids: Vec<i64> = existing_links.iter().map(|l| l.item_id).collect();
    let mut existing_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if !existing_ids.is_empty() {
        for it in item_model::Entity::find_safety()
            .filter(item_model::Column::Id.is_in(existing_ids))
            .all(db)
            .await?
        {
            existing_names.insert(it.name);
        }
    }

    // 候选 item：按名称分组去重，取每组的第一个 id（Java 逻辑）。
    let candidates = item_model::Entity::find_safety()
        .filter(item_model::Column::Id.is_in(item_id_list))
        .all(db)
        .await?;
    let mut first_by_name: std::collections::BTreeMap<String, i64> =
        std::collections::BTreeMap::new();
    for it in candidates {
        if existing_names.contains(&it.name) {
            continue;
        }
        first_by_name.entry(it.name).or_insert(it.id);
    }

    if first_by_name.is_empty() {
        return Ok(CommonResponse::new(Ok(false)));
    }

    let now = Utc::now().naive_utc();
    let mut models = Vec::with_capacity(first_by_name.len());
    for item_id in first_by_name.into_values() {
        models.push(iap_model::ActiveModel {
            version: Set(0),
            id: sea_orm::ActiveValue::NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),
            item_id: Set(item_id),
        });
    }
    iap_model::Entity::insert_many(models).exec(db).await?;
    super::binary_doc::invalidate_item_doc_cache().await;
    Ok(CommonResponse::new(Ok(true)))
}

/// `DELETE /item_common/delete/{itemId}`
///
/// 对齐 Java：按 item_id 软删 `item_area_public` 关联行（不动 item 表）。
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    iap_model::Entity::update_many()
        .col_expr(
            iap_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(iap_model::Column::ItemId.eq(id))
        .exec(db)
        .await?;
    super::binary_doc::invalidate_item_doc_cache().await;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
