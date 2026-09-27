//! sea-orm-migration 迁移体系。
//!
//! 在此之前本仓库唯一的 DDL 路径是 init_db 的 `CREATE TABLE IF NOT
//! EXISTS`——给实体加列对已存在的库是静默 no-op，没有 ALTER 路径（#141 的
//! `--check` 漂移检测只能发现、不能演进）。本 crate 引入 sea-orm-migration
//! 补上真正的迁移框架：状态表（`seaql_migrations`，落在目标 schema 内，
//! 未加限定的表名经连接 `search_path` 解析）记录每个迁移是否已应用，
//! `Migrator::up` 只执行待处理项。
//!
//! **baseline 幂等设计**：存量已部署库没有 `seaql_migrations` 表，首次运行
//! `up` 时 baseline 仍会执行——因此 baseline 的建表沿用 `CREATE TABLE IF
//! NOT EXISTS` 改写、索引沿用 `CREATE INDEX IF NOT EXISTS`，对已有对象是
//! no-op，执行完仅把 baseline 记录为 applied。全新库则一次建成 24 张表 +
//! 全部性能索引。两条路径共用同一份 DDL 来源（实体定义 + `scripts/
//! indexes_dev.sql`），与 init_db 时代严格同源。
//!
//! **演进纪律**：
//! - baseline（`m20260927_000001_baseline`）已冻结——它代表了所有存量库的
//!   当前形态，修改它等于篡改历史，会让「已应用 baseline」的库永远得不到
//!   新改动；
//! - 未来 schema 变更 = 在本 crate 新增 `mYYYYMMDD_NNNNNN_name.rs` 迁移文件
//!   （up 里写 ALTER/CREATE，down 里写逆操作），并注册进 [`Migrator::
//!   migrations`] 列表（时间序在前）；
//! - CLI 用法（workspace 根）：`cargo run -p _migration -- status` 查看状态，
//!   其余子命令见 `src/main.rs` 顶部注释。
//!
//! init_db 已改为调用 [`Migrator::up`] 完成建表与索引，`--check` 漂移检测
//! 经由本 crate 导出的 [`macro@for_each_entity`] 宏取得实体清单。

pub use sea_orm_migration::prelude::*;

mod m20260927_000001_baseline;

/// 全部 24 个实体的唯一清单，顺序即外键依赖顺序（被引用表在前：sys_user
/// → 独立主表 → 链接表 → 系统辅助表），与建表的 FOREIGN KEY 约束要求一致。
/// baseline 迁移的 up（建表）/down（逆序删表）与 init_db 的 `--check`
/// 漂移检测都经由本宏展开同一份列表——实体清单一旦手抄两份，两份迟早漂移，
/// 而「清单漂移」正是漂移检测要消灭的问题，工具自身不能先犯。
///
/// 实体写 `_database::models::...` 全路径而非调用方 use 进来的短别名：宏经
/// `#[macro_export]` 导出后会在 `_migration` 与 `_router` 两个 crate 里展开，
/// 宏体内路径按定义点（本 crate）解析，全路径让宏自包含，调用方无需再抄
/// 一遍 use 清单（`$crate` 在这里帮不上忙——实体在 `_database` 而非本
/// crate）。第一个参数是「操作宏」：本宏只是无类型的 token 转发器，把连接
/// 与上下文一并转给回调宏，各回调按自己的用途解释 `$ctx`——baseline 的
/// 建表回调收 sea-orm `Schema` 构造器、删表回调不收上下文、init_db 的漂移
/// 检测回调收 schema 名字符串。
#[macro_export]
macro_rules! for_each_entity {
    ($mac:ident, $db:expr, $ctx:expr) => {
        $mac!(
            $db,
            $ctx,
            _database::models::system::sys_user::Entity,
            _database::models::area::area::Entity,
            _database::models::icon::icon::Entity,
            _database::models::icon::icon_type::Entity,
            _database::models::item::item::Entity,
            _database::models::item::item_type::Entity,
            _database::models::tag::tag::Entity,
            _database::models::tag::tag_type::Entity,
            _database::models::marker::marker::Entity,
            _database::models::common::notice::Entity,
            _database::models::common::route::Entity,
            _database::models::common::history::Entity,
            _database::models::common::score_stat::Entity,
            _database::models::icon::icon_type_link::Entity,
            _database::models::item::item_type_link::Entity,
            _database::models::tag::tag_type_link::Entity,
            _database::models::marker::marker_item_link::Entity,
            _database::models::marker::marker_linkage::Entity,
            _database::models::area::item_area_public::Entity,
            _database::models::marker::marker_punctuate::Entity,
            _database::models::system::sys_user_archive::Entity,
            _database::models::system::sys_user_device::Entity,
            _database::models::system::sys_user_invitation::Entity,
            _database::models::system::sys_action_log::Entity,
        )
    };
}

/// 迁移注册表——时间序排列，最新的迁移追加到列表末尾（见 crate 顶部
/// 「演进纪律」）。`MigratorTrait::up/down/status` 等全部操作围绕此列表
/// 与 `seaql_migrations` 状态表展开。
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260927_000001_baseline::Migration)]
    }
}

/// 从实体生成幂等的建表 DDL：`Schema::create_table_from_entity` 产出的是裸
/// `CREATE TABLE`，这里改写为 `CREATE TABLE IF NOT EXISTS`——baseline 必须
/// 对存量库的已有表 no-op（见 crate 顶部「baseline 幂等设计」）。DDL 直接
/// 来自实体定义，不可能与代码漂移。
pub(crate) fn create_table_if_not_exists_sql<E: sea_orm::EntityTrait>(
    schema: &sea_orm::Schema,
    entity: E,
) -> String {
    let stmt = schema.create_table_from_entity(entity);
    let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder);
    sql.replacen("CREATE TABLE", "CREATE TABLE IF NOT EXISTS", 1)
}
