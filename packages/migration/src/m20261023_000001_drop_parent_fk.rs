//! 移除自引用 `parent_id` 外键。
//!
//! 数据模型（Java 与前端共有契约）用 `parent_id = -1` 表示根节点：
//! 插入根节点时 FK 要求 `parent_id` 指向已存在的行，而 `-1` 永远不存在，
//! 因此该 FK 与契约**结构上不可兼容**——新建根地区/根分类必然 500。
//! Java 侧线上库没有这条约束（历史 schema 未建 FK），Rust baseline 由
//! sea-orm 实体推断时带出了它；这里按契约移除。
//!
//! 受影响表/列：area.parent_id、icon_type.parent_id、item_type.parent_id、
//! tag_type.parent_id。幂等：约束不存在时 `DROP CONSTRAINT IF EXISTS` 为
//! no-op，可安全在已部署库与全新库上执行。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const TARGETS: &[(&str, &str)] = &[
    ("area", "fk-area-parent_id"),
    ("icon_type", "fk-icon_type-parent_id"),
    ("item_type", "fk-item_type-parent_id"),
    ("tag_type", "fk-tag_type-parent_id"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (table, constraint) in TARGETS {
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "ALTER TABLE \"{table}\" DROP CONSTRAINT IF EXISTS \"{constraint}\""
                ))
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 不回滚：重新加回 FK 会让根节点插入再次失败（契约不允许），
        // 回滚到「不可用」的状态没有意义。
        Ok(())
    }
}
