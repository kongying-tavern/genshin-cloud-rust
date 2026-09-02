pub mod app;
pub mod area;
pub mod binary_doc;
pub mod cache;
pub mod history;
pub mod icon;
pub mod icon_doc;
pub mod icon_type;
pub mod item;
pub mod item_common;
pub mod item_doc;
pub mod item_type;
pub mod marker;
pub mod marker_doc;
pub mod marker_link;
pub mod marker_link_doc;
pub mod notice;
pub mod res;
// 已弃用（对齐决策，见 docs/zh-chs/guides/sync-with-java-roadmap.md）：
// Java 的 route / punctuate / punctuate_audit 三域业务不在本后端实现。
// pub mod route;
pub mod score;
pub mod tag;
pub mod tag_doc;
pub mod tag_type;

use std::collections::HashSet;

use anyhow::Result;
use sea_orm::prelude::*;

use _database::models::system::sys_user as sys_user_model;
use _utils::db_operations::SafeEntityTrait;

/// 按用户 id 批量查询，构建前端 `Record<string, SysUserSmallVo>` 契约的
/// `{id: {id, username, nickname}}` map，用于 `CommonResponse.users`。
pub(crate) async fn sys_user_map(
    db: &sea_orm::DatabaseConnection,
    user_ids: &HashSet<i64>,
) -> Result<serde_json::Value> {
    let mut users = serde_json::Map::new();
    if user_ids.is_empty() {
        return Ok(serde_json::Value::Object(users));
    }
    let mut ids: Vec<i64> = user_ids.iter().copied().collect();
    ids.sort_unstable();
    let mut rows: Vec<sys_user_model::Model> = Vec::new();
    for chunk in ids.chunks(1000) {
        rows.extend(
            sys_user_model::Entity::find_safety()
                .filter(sys_user_model::Column::Id.is_in(chunk))
                .all(db)
                .await?,
        );
    }
    for u in rows {
        users.insert(
            u.id.to_string(),
            serde_json::json!({
                "id": u.id,
                "username": u.username,
                "nickname": u.nickname,
            }),
        );
    }
    Ok(serde_json::Value::Object(users))
}
