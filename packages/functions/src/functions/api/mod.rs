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
// 已弃用（对齐决策，见 docs/zh-Hans/guides/sync-with-java-roadmap.md）：
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

/// 父级 is_final（末端标志）维护的统一实现：子级增删后刷新父级派生字段。
///
/// 语义（四域统一）：
/// - `update_many` 批量直写——is_final 是由子级数量推导的**派生字段**，
///   维护它不参与乐观锁：并发子级新增不会因版本号互相冲突，也不会让
///   正在编辑父级实体的用户撞「已更新」；
/// - 同时刷新 update_time（子树变更也是父级变更，审计时间应前进），
///   但不动 version / updater_id（前者见上，后者由真实编辑者语义持有）；
/// - 过滤 del_flag = false（对齐原 find_safety_by_id 语义：软删父级不维护）；
/// - parent_id <= 0（根级哨兵，如 -1/0）直接 Ok 跳过。
///
/// 各域以薄包装传入自己的 Entity 与列（见 area/icon_type/item_type/tag_type
/// 的 set_parent_is_final），调用方错误处理策略不变。
// 列参数（id/is_final/update_time/del_flag）由各域显式传入，换取 helper
// 对任意实体泛化；参数数超 clippy 默认上限，与 action_log 等处同样放行。
#[allow(clippy::too_many_arguments)]
pub(super) async fn set_derived_is_final<E>(
    db: &sea_orm::DatabaseConnection,
    // sea-orm 2.0 的 update_many() 是关联函数，实体实例不参与调用；
    // 但 E 无法从 E::Column 参数反推（投影不可逆向推断），必须显式传入
    // 以锚定泛型，故保留下划线参数。
    _entity: E,
    id_col: E::Column,
    is_final_col: E::Column,
    update_time_col: E::Column,
    del_flag_col: E::Column,
    parent_id: i64,
    is_final: bool,
) -> Result<()>
where
    E: EntityTrait,
{
    if parent_id <= 0 {
        return Ok(());
    }
    // update_time 以 Option<NaiveDateTime> 绑定，与四域实体统一的
    // Option<DateTime> 可空列定义对齐（值恒为 Some）。
    E::update_many()
        .col_expr(is_final_col, Expr::value(is_final))
        .col_expr(
            update_time_col,
            Expr::value(Some(chrono::Utc::now().naive_utc())),
        )
        .filter(id_col.eq(parent_id))
        .filter(del_flag_col.eq(false))
        .exec(db)
        .await?;
    Ok(())
}
