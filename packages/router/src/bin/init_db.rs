//! Idempotent schema initializer for local/e2e development.
//!
//! 建表与索引已收编进 sea-orm-migration 迁移体系（`packages/migration`）：
//! 本 bin 的主流程是 建连接（含 ensure schema）→ `_migration::Migrator::up`
//! （状态表 `seaql_migrations` + 幂等 baseline，天然可重复执行）→ `--check`
//! 漂移检测 / 播种 dev admin。baseline 的建表 DDL 直接来自 sea-orm 实体
//! 定义、索引来自 `scripts/indexes_dev.sql`（均已编进 `_migration`），
//! 与实体代码不可能漂移。
//!
//! **如何新增迁移**：在 `packages/migration/src/` 加 `mYYYYMMDD_NNNNNN_
//! name.rs`（up 写 ALTER/CREATE，down 写逆操作）并注册进 `Migrator::
//! migrations`。baseline（m20260927_000001）已冻结，禁止修改——存量库已
//! 记录它为 applied，改它等于篡改历史。查看迁移状态：
//!   cargo run -p _migration -- status
//! （其余子命令 up/down/fresh/refresh 见其 main.rs 顶部注释，均面向
//! dev 环境。）
//!
//! **漂移检测模式（`--check`，#141 引入）**：在迁移跑完后，对全部 24 个
//! 实体逐一把「实体声明的列集合」与库侧 `information_schema.columns`
//! 比对并打印中文报告（实体清单单一事实源是 `_migration::for_each_entity!`
//! 宏，与 baseline 建表共用同一份列表）：
//!
//! - 实体有、库无（missing）→ 硬失败，退出码 1（SELECT 列清单缺列，
//!   运行期 500 面）；
//! - 库有、实体无（extra）→ 仅警告，不影响退出码（sea-orm SELECT 用
//!   显式列清单，多余库列不破坏读取，只提示历史漂移）。
//!
//! `--check` 不播种 dev admin（检测不写业务数据），也绝不做 ALTER——
//! 补齐/回退由运维执行。建议 CI/运维在部署前对目标库跑一次，把漂移从
//! 运行期 500 提前到部署期：
//!
//!   cargo run --bin init_db -- --check
//!
//! Usage (from the workspace root):
//!   cargo run --bin init_db            # 迁移 + 播种 dev admin
//!   cargo run --bin init_db -- --check # 迁移 + 漂移检测（不播种）
//!
//! DB connection comes from the standard `DB_*` env vars (see `.env.example`);
//! `scripts/init_db.py` wraps this for the e2e/dev workflow (透传本 bin 的
//! 全部参数，含 `--check`).

use anyhow::{Context, Result, bail};

use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, IdenStatic, Iterable,
    QueryFilter, Statement,
};

use std::collections::BTreeSet;

use _database::default_schema;

use _database::models::system::sys_user as sys_user_entity;

// `Migrator::up` 是 MigratorTrait 的关联函数，trait 需在作用域内。
use _migration::{Migrator, MigratorTrait};

/// `--check` 模式的逐表比对入口：与 baseline 建表共用
/// `_migration::for_each_entity!` 清单（全路径实体、`#[macro_export]`
/// 导出），保证「检测到的 24 张表」与「实际建出来的 24 张表」永远同源
/// 同序。
macro_rules! check_drift {
    ($db:expr, $schema:expr, $($entity:expr),+ $(,)?) => {{
        let mut report = Vec::new();
        $(
            // stringify! 会把 `::` 规整成 " :: "，去掉空格让报告里的实体
            // 路径与源码写法一致（...models::marker::marker::Entity）
            let label = stringify!($entity).replace(" :: ", "::");
            report.push(
                check_entity_drift($db, $schema, label, $entity)
                    .await
                    .with_context(|| format!("check drift for {}", stringify!($entity)))?,
            );
        )+
        report
    }};
}

#[tokio::main]
async fn main() -> Result<()> {
    // CLI：无参数 = 现有行为（迁移 + 播种 dev admin）；--check = 漂移检测
    // 模式（迁移后比对列集合，不播种，见文件头）。其余参数一律报错拒绝
    // 而不是静默忽略——初始化工具「猜错意图」比报错危险。
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    let check_mode = match cli_args.as_slice() {
        [] => false,
        [arg] if arg == "--check" => true,
        args => bail!(
            "未知参数 {args:?}：用法 init_db [--check]（--check = 漂移检测模式，详见文件头注释）"
        ),
    };

    let schema = default_schema();
    let db = _database::connect_standalone_pg().await?;

    // 迁移体系接管建表与索引（替代旧的 ensure_tables! + on-demand 探测 +
    // ensure_indexes 三段）：状态表 + IF NOT EXISTS baseline 对任意存量库
    // 幂等——旧库首跑对已有对象 no-op 后记录 applied，全新库一次建成，
    // 重复执行永远安全。
    Migrator::up(&db, None).await.context("apply migrations")?;
    println!("Schema ready ({schema}): migrations applied, indexes ensured");

    if check_mode {
        // 检测模式不播种 dev admin：检测是对目标库结构的只读判定，不应写
        // 业务数据；播种只在普通模式分支执行。
        let report = _migration::for_each_entity!(check_drift, &db, schema.as_str());
        let has_missing = print_drift_report(&report);
        if has_missing {
            // 漂移是「检测结果」而非异常链：完整报告已打印到 stdout，再走
            // anyhow 只会把摘要重复打印一遍——直接以退出码表达部署阻断。
            std::process::exit(1);
        }
        return Ok(());
    }

    ensure_admin_account(&db).await?;
    Ok(())
}

/// Ensure a dev admin account exists (dev-only bootstrap). Prints the
/// credentials to stdout when it creates one; override via
/// `INIT_ADMIN_USERNAME` / `INIT_ADMIN_PASSWORD`. 已有 admin（按角色查询）
/// 则跳过——迁移体系的幂等语义下重复跑本 bin 不会重复造号。
async fn ensure_admin_account(db: &sea_orm::DatabaseConnection) -> Result<()> {
    let username = std::env::var("INIT_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
    let password = std::env::var("INIT_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".into());

    let existing = sys_user_entity::Entity::find()
        .filter(sys_user_entity::Column::RoleId.eq(_utils::types::SystemUserRole::Admin))
        .one(db)
        .await?;
    if existing.is_some() {
        println!(
            "Admin account already present ({}); nothing to seed",
            existing.map(|u| u.username).unwrap_or_default()
        );
        return Ok(());
    }

    let now = chrono::Utc::now().naive_utc();
    sys_user_entity::Entity::insert(sys_user_entity::ActiveModel {
        version: Set(1),
        id: sea_orm::ActiveValue::NotSet,
        // 审计字段：新增时 create/update 两组时间设置。引导用户是库里第一行，
        // 尚无任何可引用的操作者——creator/updater 置 NULL（Some(0) 会撞
        // fk-sys-user-creator_id 自引用外键：空表不存在 id=0 的行，全新库
        // 播种必失败）。
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        username: Set(username.clone()),
        password: Set(_utils::bcrypt::generate_storage_password(&password)?),
        nickname: Set(Some("Dev Admin".into())),
        qq: Set(None),
        phone: Set(None),
        logo: Set(None),
        role_id: Set(_utils::types::SystemUserRole::Admin),
        access_policy: Set(None),
        remark: Set(Some("Auto-seeded by init_db (dev only)".into())),
    })
    .exec(db)
    .await?;

    println!(
        "Seeded dev admin account: username={username} password={password} \
         (dev only — override via INIT_ADMIN_USERNAME / INIT_ADMIN_PASSWORD)"
    );
    Ok(())
}

// ── 漂移检测（--check）────────────────────────────────────────────────────

/// 单表的漂移比对结果。missing/extra 由两侧 BTreeSet 差集算出，天然有序：
/// 同样的漂移两次运行输出完全一致，便于 CI 日志对比与回归追踪。
struct TableDrift {
    /// 实体路径文本（诊断定位用，如 `_database::models::marker::marker::Entity`）
    entity: String,
    /// 表名（与实体 `table_name` 同源）
    table: String,
    /// 实体声明、库中缺失 —— 硬失败面：sea-orm 的 SELECT 用实体列清单生成，
    /// 缺列直接 SQL 报错，正是运行期 500 的来源
    missing: Vec<String>,
    /// 库中存在、实体未声明 —— 仅警告：SELECT 列清单来自实体，多余库列
    /// 不破坏读取，只提示历史上发生过实体瘦身/手工改库
    extra: Vec<String>,
}

/// 实体侧「表名 + 声明的全部列名」。列名取 sea-orm 为 `Column` 枚举生成的
/// `IdenStatic::as_str`（变体名 → snake_case 的真实 DB 列名，支持
/// `column_name` 属性覆盖）——与 `create_table_from_entity` 生成 DDL 用的是
/// 同一个标识符来源，实体侧不可能出现「DDL 里没有的列」，比对基准因此与
/// 建表路径严格同源。
fn entity_columns<E: EntityTrait>(entity: E) -> (String, BTreeSet<String>) {
    (
        entity.table_name().to_owned(),
        E::Column::iter().map(|c| c.as_str().to_owned()).collect(),
    )
}

/// 库侧列名：参数化查询 `information_schema.columns`，schema/表名都走绑定
/// 参数，杜绝任何 SQL 拼接面。schema 名来自 `default_schema()`（已有 bare
/// identifier 校验回退），表名即 `entity_columns` 的返回值——两侧比对的是
/// 同一张表的同一套命名，不存在「实体叫 A、库里查 B」的错位。
async fn db_columns(
    db: &sea_orm::DatabaseConnection,
    schema: &str,
    table: &str,
) -> Result<BTreeSet<String>> {
    let stmt = Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT column_name FROM information_schema.columns \
         WHERE table_schema = $1 AND table_name = $2",
        [schema.into(), table.into()],
    );
    let rows = db.query_all_raw(stmt).await?;
    let mut columns = BTreeSet::new();
    for row in rows {
        let name: String = row
            .try_get("", "column_name")
            .with_context(|| format!("read column_name for table {table}"))?;
        columns.insert(name);
    }
    Ok(columns)
}

/// 比对单个实体与库结构，产出该表的漂移结果。
async fn check_entity_drift<E: EntityTrait>(
    db: &sea_orm::DatabaseConnection,
    schema: &str,
    entity_label: String,
    entity: E,
) -> Result<TableDrift> {
    let (table, entity_cols) = entity_columns(entity);
    let db_cols = db_columns(db, schema, &table).await?;
    Ok(TableDrift {
        entity: entity_label,
        missing: entity_cols.difference(&db_cols).cloned().collect(),
        extra: db_cols.difference(&entity_cols).cloned().collect(),
        table,
    })
}

/// 打印中文漂移报告，返回是否存在缺失列（= 硬失败 / 退出码 1 的依据）。
/// extra 不参与退出码：退出码的语义是「此库能否安全跑当前代码」，多余库列
/// 不影响读取；missing 则是必然的运行期 500，必须阻断部署。
fn print_drift_report(report: &[TableDrift]) -> bool {
    println!(
        "── 漂移检测报告（实体定义 ↔ 目标库结构，共 {} 张表）──",
        report.len()
    );
    let mut has_missing = false;
    for t in report {
        if !t.missing.is_empty() {
            has_missing = true;
            println!(
                "  missing  {}：实体有、库无 {} 列（SELECT 将缺列 → 运行期 500 面）[{}]",
                t.table,
                t.missing.len(),
                t.entity
            );
            for c in &t.missing {
                println!("      - {c}");
            }
            // 同表既有缺失又有多余时，多余列并进 missing 分支打印，仍只是警告
            if !t.extra.is_empty() {
                println!(
                    "      另有库有、实体无 {} 列（仅警告）：{}",
                    t.extra.len(),
                    t.extra.join(", ")
                );
            }
        } else if !t.extra.is_empty() {
            println!(
                "  extra    {}：库有、实体无 {} 列（仅警告，不影响读取）[{}]",
                t.table,
                t.extra.len(),
                t.entity
            );
            for c in &t.extra {
                println!("      - {c}");
            }
        } else {
            println!("  ok       {}", t.table);
        }
    }

    let missing_tables = report.iter().filter(|t| !t.missing.is_empty()).count();
    let missing_cols: usize = report.iter().map(|t| t.missing.len()).sum();
    let extra_tables = report.iter().filter(|t| !t.extra.is_empty()).count();
    let extra_cols: usize = report.iter().map(|t| t.extra.len()).sum();

    if has_missing {
        println!(
            "汇总：{missing_tables}/{} 张表缺失列（共 {missing_cols} 列），\
             {extra_tables} 张表有多余列（共 {extra_cols} 列）",
            report.len()
        );
        println!(
            "修复指引：在目标库补齐缺失列，或回退实体改动；\
             ALTER 由运维执行——本工具不做 DDL 变更。"
        );
    } else if extra_cols > 0 {
        println!(
            "汇总：无缺失列；{extra_tables} 张表有多余列\
             （共 {extra_cols} 列，历史漂移提示，不阻断部署）"
        );
        println!("schema 漂移检测通过：全部 {} 张表无缺失列。", report.len());
    } else {
        println!(
            "schema 漂移检测通过：全部 {} 张表列集合与实体定义一致。",
            report.len()
        );
    }
    has_missing
}

/// 列名提取方法的正确性证明（无 DB 也能跑）：`entity_columns` 必须产出真实
/// DB 列名（snake_case），而不是 `Column` 枚举的变体名（如 `MarkerTitle`）。
/// 断言逐列对照 `_database::models` 里各实体的字段定义。
#[cfg(test)]
mod tests {
    use super::*;

    // sys_user_entity 经 `use super::*` 取得；这里只补测试专属的两个别名。
    use _database::models::{
        marker::marker as marker_entity, marker::marker_linkage as marker_linkage_entity,
    };

    #[test]
    fn marker_entity_columns_are_real_db_names() {
        let (table, cols) = entity_columns(marker_entity::Entity);
        assert_eq!(table, "marker");
        // 对照 packages/database/src/models/marker/marker.rs 的全部字段
        for expected in [
            "version",
            "id",
            "create_time",
            "update_time",
            "creator_id",
            "updater_id",
            "del_flag",
            "marker_stamp",
            "marker_title",
            "position",
            "content",
            "picture",
            "marker_creator_id",
            "picture_creator_id",
            "video_path",
            "refresh_time",
            "hidden_flag",
            "extra",
        ] {
            assert!(
                cols.contains(expected),
                "marker 应含列 {expected}，实际列集合 {cols:?}"
            );
        }
        // 变体名绝不能出现——证明拿到的是列名而非枚举名
        assert!(!cols.contains("MarkerTitle"));
        assert!(!cols.contains("HiddenFlag"));
    }

    #[test]
    fn sys_user_entity_columns_are_real_db_names() {
        let (table, cols) = entity_columns(sys_user_entity::Entity);
        assert_eq!(table, "sys_user");
        for expected in [
            "version",
            "id",
            "create_time",
            "update_time",
            "creator_id",
            "updater_id",
            "del_flag",
            "username",
            "password",
            "nickname",
            "qq",
            "phone",
            "logo",
            "role_id",
            "access_policy",
            "remark",
        ] {
            assert!(
                cols.contains(expected),
                "sys_user 应含列 {expected}，实际列集合 {cols:?}"
            );
        }
        assert!(!cols.contains("Username"));
    }

    #[test]
    fn marker_linkage_entity_columns_are_real_db_names() {
        let (table, cols) = entity_columns(marker_linkage_entity::Entity);
        assert_eq!(table, "marker_linkage");
        // 链接表也覆盖一组：证明方法对任意实体成立，不止主表
        for expected in ["version", "id", "group_id", "from_id", "to_id"] {
            assert!(
                cols.contains(expected),
                "marker_linkage 应含列 {expected}，实际列集合 {cols:?}"
            );
        }
    }
}
