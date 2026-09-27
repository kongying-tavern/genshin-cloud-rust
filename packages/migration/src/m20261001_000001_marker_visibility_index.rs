//! m20261001_000001_marker_visibility_index —— 首个正式演进迁移。
//!
//! 这是 baseline 之后第一个 schema 变更，也是迁移纪律的实战首演：
//! baseline 已冻结（见 lib.rs「演进纪律」），任何后续变更都以本文件这样
//! 的新迁移落地——注册进 [`crate::Migrator`]，存量库下次 `up` 只应用增量。
//!
//! 内容：marker 表的复合索引 `(del_flag, hidden_flag)`。marker 是全库最大
//! 的表，所有读路径都按 `del_flag = false AND hidden_flag IN (调用者可见
//! 集合)` 过滤（SafeEntityTrait 的软删过滤 + hidden_flag 可见性过滤两层
//! 叠加），这条复合索引服务最热的过滤读路径。
//!
//! up/down 均幂等（IF NOT EXISTS / IF EXISTS），索引名带 schema 限定——
//! 与 baseline 的索引应用同一套前缀改写（`default_schema()` 解析，
//! DB_SCHEMA 可覆盖）。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = _database::default_schema();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"CREATE INDEX IF NOT EXISTS "idx_marker_del_flag_hidden_flag"
                   ON "{schema}"."marker" ("del_flag", "hidden_flag")"#
            ))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = _database::default_schema();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"DROP INDEX IF EXISTS "{schema}"."idx_marker_del_flag_hidden_flag""#
            ))
            .await?;
        Ok(())
    }
}
