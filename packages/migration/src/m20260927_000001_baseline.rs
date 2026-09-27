//! m20260927_000001_baseline —— 初始基线迁移（**已冻结，禁止修改**）。
//!
//! baseline 是把 init_db 时代的「实体反向 DDL 建表 + `scripts/indexes_dev.sql`
//! 性能索引」原样收编为第一个迁移。存量已部署库没有迁移状态表，首跑时
//! baseline 仍会执行，因此它的每条语句都必须幂等：建表走 `CREATE TABLE IF
//! NOT EXISTS` 改写、索引走文件自带的 `CREATE INDEX IF NOT EXISTS`——对
//! 已有对象 no-op，执行完仅记录 applied（见 crate 顶部「baseline 幂等
//! 设计」）。未来 schema 变更一律新增迁移文件，不改这里。

use sea_orm_migration::prelude::*;
// drop_table 对「具体」实体类型调 `table_name()`（EntityName 的方法）——
// 泛型上下文里 EntityTrait 边界会顺带放开父 trait 方法，具体类型则要求
// trait 显式在作用域内，故单独引入。
use sea_orm_migration::sea_orm::EntityName;

/// 单回调宏：对清单里的每个实体执行幂等建表。`IF NOT EXISTS` 改写在
/// [`crate::create_table_if_not_exists_sql`]（从 init_db 的 ensure_tables!
/// 原样搬来）；`$ctx` 收 sea-orm `Schema` 构造器。
macro_rules! ensure_table {
    ($conn:expr, $schema:expr, $($entity:expr),+ $(,)?) => {{
        let mut created = 0usize;
        $(
            let sql = $crate::create_table_if_not_exists_sql($schema, $entity);
            $conn.execute_unprepared(&sql).await.map_err(|e| {
                DbErr::Custom(format!("create table for {}: {e}", stringify!($entity)))
            })?;
            created += 1;
        )+
        created
    }};
}

/// 单回调宏：逆序删除清单里的全部表。清单顺序是外键依赖序（被引用表在
/// 前），逆序删除保证任何时刻都不会先删仍被引用的表——出现外键报错即
/// 说明清单顺序本身有错，是有价值的信号，因此不加 CASCADE 掩盖。性能
/// 索引随表消亡，无需单独处理；`$ctx` 不使用。
macro_rules! drop_table {
    ($conn:expr, $_ctx:expr, $($entity:expr),+ $(,)?) => {{
        let mut tables = Vec::new();
        $(
            tables.push(($entity.table_name().to_owned(), stringify!($entity)));
        )+
        for (table, entity) in tables.into_iter().rev() {
            $conn.execute_unprepared(&format!(r#"DROP TABLE IF EXISTS "{table}""#))
                .await
                .map_err(|e| DbErr::Custom(format!("drop table {table} ({entity}): {e}")))?;
        }
    }};
}

/// baseline 迁移本体。`DeriveMigrationName` 以本模块名作为迁移名
/// （`m20260927_000001_baseline`），即状态表里记录的 version 值。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// ① 按外键依赖顺序对 24 个实体建表（幂等）；② 应用 `scripts/
    /// indexes_dev.sql` 性能索引（幂等）。sea-orm-migration 在 Postgres 上
    /// 默认把整个迁移包进单个事务，DDL 全有或全无。
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let created = crate::for_each_entity!(
            ensure_table,
            conn,
            &sea_orm::Schema::new(sea_orm::DbBackend::Postgres)
        );
        apply_indexes(conn).await?;
        tracing::info!("Baseline ready: {created} tables ensured, performance indexes applied");
        Ok(())
    }

    /// 逆序删掉全部表（含索引）。仅面向 dev 环境——本地重来、验证
    /// fresh/refresh 语义用；生产库对 baseline 执行 down 等于删库，运维
    /// 纪律上绝不执行。
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        crate::for_each_entity!(drop_table, manager.get_connection(), ());
        Ok(())
    }
}

/// 应用 `scripts/indexes_dev.sql` 的性能索引（幂等），从 init_db 的
/// ensure_indexes 原样搬来。文件经 include_str! 编进二进制，与运维 psql
/// 手跑的是同一份，不可能漂移；语句剥掉整行注释后按分号切分逐条执行。
/// 文件里的表名硬编码 `genshin_map.` 前缀，执行前改写为 `default_schema()`
/// 解析出的 schema 名（DB_SCHEMA 可覆盖，带 bare-identifier 校验回退）。
/// 迁移的 up 拿不到外部参数，schema 名只能从 env 解析——与 init_db 时代
/// 行为一致。
async fn apply_indexes(conn: &impl ConnectionTrait) -> Result<(), DbErr> {
    let schema = _database::default_schema();
    // include_str! 相对本文件（packages/migration/src/）定位：三级上溯
    // 到仓库根再进 scripts/。
    let sql = include_str!("../../../scripts/indexes_dev.sql")
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("--")
        })
        .collect::<Vec<_>>()
        .join(" ");
    for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let stmt = stmt.replace("genshin_map.", &format!("{schema}."));
        conn.execute_unprepared(&stmt)
            .await
            .map_err(|e| DbErr::Custom(format!("create index: {stmt}: {e}")))?;
    }
    Ok(())
}
