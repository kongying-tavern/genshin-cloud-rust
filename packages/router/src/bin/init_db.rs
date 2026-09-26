//! Idempotent schema initializer for local/e2e development.
//!
//! Creates the schema (default `genshin_map`, override via `DB_SCHEMA`) and
//! every table from the sea-orm models (`CREATE TABLE IF NOT EXISTS`), then
//! exits. DDL is generated from the entity definitions, so it can never drift
//! from the code.
//!
//! **On-demand mode**: when all 24 tables already exist the CREATE pass is
//! skipped entirely ("schema already up to date"). Afterwards it applies the
//! performance indexes from `scripts/indexes_dev.sql` (idempotent), then
//! ensures a dev admin account exists and prints the credentials to stdout.
//!
//! **漂移检测模式（`--check`）**：本仓库没有迁移体系——唯一的 DDL 路径就是
//! `CREATE TABLE IF NOT EXISTS`，对已存在的表是静默 no-op。给实体加列后，
//! 已部署的库不会跟着变，实体定义与真实库结构的漂移只能在运行期以 500
//! 暴露（审计认定的最高演进风险）。`--check` 在照常执行幂等的建
//! schema/建表/应用索引之后，对全部 24 个实体逐一把「实体声明的列集合」
//! 与库侧 `information_schema.columns` 比对并打印中文报告：
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
//! The index SQL file is also the ops source of truth for the production
//! database — run it manually there (see the file header):
//!   psql ".../genshin_map" -f scripts/indexes_dev.sql
//!
//! Usage (from the workspace root):
//!   cargo run --bin init_db            # 建表 + 索引 + 播种 dev admin
//!   cargo run --bin init_db -- --check # 建表 + 索引 + 漂移检测（不播种）
//!
//! DB connection comes from the standard `DB_*` env vars (see `.env.example`);
//! `scripts/init_db.py` wraps this for the e2e/dev workflow (透传本 bin 的
//! 全部参数，含 `--check`).

use anyhow::{Context, Result, bail};

use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectOptions, ConnectionTrait, DbBackend, EntityTrait,
    IdenStatic, Iterable, QueryFilter, Schema, Statement,
};

use std::collections::BTreeSet;

use _database::{default_schema, encode_url_component};

use _database::models::{
    area::area as area_entity, area::item_area_public as item_area_public_entity,
    common::history as history_entity, common::notice as notice_entity,
    common::route as route_entity, common::score_stat as score_stat_entity,
    icon::icon as icon_entity, icon::icon_type as icon_type_entity,
    icon::icon_type_link as icon_type_link_entity, item::item as item_entity,
    item::item_type as item_type_entity, item::item_type_link as item_type_link_entity,
    marker::marker as marker_entity, marker::marker_item_link as marker_item_link_entity,
    marker::marker_linkage as marker_linkage_entity,
    marker::marker_punctuate as marker_punctuate_entity,
    system::sys_action_log as sys_action_log_entity, system::sys_user as sys_user_entity,
    system::sys_user_archive as sys_user_archive_entity,
    system::sys_user_device as sys_user_device_entity,
    system::sys_user_invitation as sys_user_invitation_entity, tag::tag as tag_entity,
    tag::tag_type as tag_type_entity, tag::tag_type_link as tag_type_link_entity,
};

/// Create every table with `CREATE TABLE IF NOT EXISTS` (idempotent). The
/// DDL comes straight from the entity definitions, so it can't drift.
macro_rules! ensure_tables {
    ($db:expr, $schema:expr, $($entity:expr),+ $(,)?) => {{
        let mut created = 0usize;
        $(
            let stmt = $schema.create_table_from_entity($entity);
            let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder);
            // sea-orm emits plain CREATE TABLE; make it idempotent.
            let sql = sql.replacen("CREATE TABLE", "CREATE TABLE IF NOT EXISTS", 1);
            $db.execute_unprepared(&sql)
                .await
                .with_context(|| format!("create table for {}", stringify!($entity)))?;
            created += 1;
        )+
        created
    }};
}

/// 全部 24 个实体的唯一清单，顺序即外键依赖顺序（被引用表在前：sys_user
/// → 独立主表 → 链接表 → 系统辅助表），与建表的 FOREIGN KEY 约束要求一致。
/// 建表（`ensure_tables!`）与漂移检测（`check_drift!`）都经由本宏展开同一
/// 份列表——实体清单一旦手抄两份，两份迟早漂移，而「清单漂移」正是
/// `--check` 要消灭的问题，工具自身不能先犯。第二个参数随回调宏而异：
/// `ensure_tables!` 收 `Schema` 构造器，`check_drift!` 收 schema 名字符串
/// （宏只是无类型的 token 转发，各回调按自己的用途解释）。
macro_rules! for_each_entity {
    ($mac:ident, $db:expr, $ctx:expr) => {
        $mac!(
            $db,
            $ctx,
            sys_user_entity::Entity,
            area_entity::Entity,
            icon_entity::Entity,
            icon_type_entity::Entity,
            item_entity::Entity,
            item_type_entity::Entity,
            tag_entity::Entity,
            tag_type_entity::Entity,
            marker_entity::Entity,
            notice_entity::Entity,
            route_entity::Entity,
            history_entity::Entity,
            score_stat_entity::Entity,
            icon_type_link_entity::Entity,
            item_type_link_entity::Entity,
            tag_type_link_entity::Entity,
            marker_item_link_entity::Entity,
            marker_linkage_entity::Entity,
            item_area_public_entity::Entity,
            marker_punctuate_entity::Entity,
            sys_user_archive_entity::Entity,
            sys_user_device_entity::Entity,
            sys_user_invitation_entity::Entity,
            sys_action_log_entity::Entity,
        )
    };
}

/// `--check` 模式的逐表比对入口：与建表共用 `for_each_entity!` 清单，
/// 保证「检测到的 24 张表」与「实际建出来的 24 张表」永远同源同序。
macro_rules! check_drift {
    ($db:expr, $schema:expr, $($entity:expr),+ $(,)?) => {{
        let mut report = Vec::new();
        $(
            // stringify! 会把 `::` 规整成 " :: "，去掉空格让报告里的实体
            // 路径与源码写法一致（marker_entity::Entity）
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
    // CLI：无参数 = 现有行为（建表 + 索引 + 播种 dev admin）；--check =
    // 漂移检测模式（建表 + 索引后比对列集合，不播种，见文件头）。其余
    // 参数一律报错拒绝而不是静默忽略——初始化工具「猜错意图」比报错危险。
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    let check_mode = match cli_args.as_slice() {
        [] => false,
        [arg] if arg == "--check" => true,
        args => bail!(
            "未知参数 {args:?}：用法 init_db [--check]（--check = 漂移检测模式，详见文件头注释）"
        ),
    };

    let db_port = std::env::var("DB_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(5432);
    let schema = default_schema();
    // Credentials are percent-encoded (same as the main binary) so reserved
    // characters in the password can't break the URL.
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        encode_url_component(
            &std::env::var("DB_USERNAME").unwrap_or_else(|_| "genshin_map".into())
        ),
        encode_url_component(&std::env::var("DB_PASSWORD").unwrap_or_default()),
        std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()),
        db_port,
        encode_url_component(
            &std::env::var("DB_DATABASE").unwrap_or_else(|_| "genshin_map".into())
        ),
    );
    // Entities carry no schema qualifier; unqualified DDL/queries land in the
    // schema resolved through the connection `search_path`.
    let mut opt = ConnectOptions::new(url.clone());
    opt.set_schema_search_path(schema.clone());
    let db = sea_orm::Database::connect(opt)
        .await
        .with_context(|| format!("connect to {url}"))?;

    db.execute_unprepared(&format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#))
        .await
        .context("create schema")?;

    // On-demand: skip the CREATE pass entirely when the schema already
    // exists (probe a representative table).
    let schema_present = db
        .execute_unprepared(&format!(r#"SELECT 1 FROM "{schema}".sys_user LIMIT 1"#))
        .await
        .is_ok();

    if schema_present {
        println!("Schema already up to date ({schema})");
    } else {
        let schema_obj = Schema::new(DbBackend::Postgres);
        // Order matters for the FOREIGN KEY constraints: referenced tables
        // must exist first. sys_user → standalone masters (area is
        // self-referencing via parent_id, which is fine) → link tables →
        // system aux tables.
        let created = for_each_entity!(ensure_tables, db, schema_obj);
        println!("Schema ready: {created} tables ensured in {schema}");
    }

    // Performance indexes (idempotent). Always runs — also on the on-demand
    // path, where tables already exist but the indexes may not. --check 同样
    // 照常执行：检测的前提是先走完与普通模式一致的幂等 ensure 路径，全新
    // 库也要能一次跑到比对这一步。
    ensure_indexes(&db, &schema).await?;

    if check_mode {
        // 检测模式不播种 dev admin：检测是对目标库结构的只读判定，不应写
        // 业务数据；播种只在普通模式分支执行。
        let report = for_each_entity!(check_drift, &db, schema.as_str());
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

/// Apply the performance indexes from `scripts/indexes_dev.sql`
/// (`CREATE INDEX IF NOT EXISTS`, idempotent). The same file is executed
/// manually on the production database by ops; see its header comment.
/// The file is embedded via `include_str!` so the ops copy and the binary
/// can never drift apart. The statements are schema-qualified
/// (`genshin_map.`); the qualifier is rewritten to the configured schema
/// before execution.
async fn ensure_indexes(db: &sea_orm::DatabaseConnection, schema: &str) -> Result<()> {
    // Strip full-line comments, then execute statement by statement.
    // Path is relative to this file (packages/router/src/bin/).
    let sql = include_str!("../../../../scripts/indexes_dev.sql")
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("--")
        })
        .collect::<Vec<_>>()
        .join(" ");
    for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let stmt = stmt.replace("genshin_map.", &format!("{schema}."));
        db.execute_unprepared(&stmt)
            .await
            .with_context(|| format!("create index: {stmt}"))?;
    }
    println!("Indexes ensured (scripts/indexes_dev.sql)");
    Ok(())
}

/// Ensure a dev admin account exists (dev-only bootstrap). Prints the
/// credentials to stdout when it creates one; override via
/// `INIT_ADMIN_USERNAME` / `INIT_ADMIN_PASSWORD`.
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
    /// 实体路径文本（诊断定位用，如 `marker_entity::Entity`）
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
