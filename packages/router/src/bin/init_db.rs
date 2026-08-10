//! Idempotent schema initializer for local/e2e development.
//!
//! Creates the `genshin_map` schema and every table from the sea-orm models
//! (`CREATE TABLE IF NOT EXISTS`), then exits. DDL is generated from the
//! entity definitions, so it can never drift from the code.
//!
//! **On-demand mode**: when all 24 tables already exist the CREATE pass is
//! skipped entirely ("schema already up to date"). Afterwards it applies the
//! performance indexes from `scripts/indexes_dev.sql` (idempotent), then
//! ensures a dev admin account exists and prints the credentials to stdout.
//!
//! The index SQL file is also the ops source of truth for the production
//! database — run it manually there (see the file header):
//!   psql ".../genshin_map" -f scripts/indexes_dev.sql
//!
//! Usage (from the workspace root):
//!   cargo run --bin init_db
//!
//! DB connection comes from the standard `DB_*` env vars (see `.env.example`);
//! `scripts/init_db.py` wraps this for the e2e/dev workflow.

use anyhow::{Context, Result};

use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, Database, DbBackend, EntityTrait, QueryFilter,
    Schema,
};

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

#[tokio::main]
async fn main() -> Result<()> {
    let db_port = std::env::var("DB_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(5432);
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        std::env::var("DB_USERNAME").unwrap_or_else(|_| "genshin_map".into()),
        std::env::var("DB_PASSWORD").unwrap_or_default(),
        std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()),
        db_port,
        std::env::var("DB_DATABASE").unwrap_or_else(|_| "genshin_map".into()),
    );
    let db = Database::connect(&url)
        .await
        .with_context(|| format!("connect to {url}"))?;

    db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS genshin_map")
        .await
        .context("create schema")?;

    // On-demand: skip the CREATE pass entirely when the schema already
    // exists (probe a representative table).
    let schema_present = db
        .execute_unprepared("SELECT 1 FROM genshin_map.sys_user LIMIT 1")
        .await
        .is_ok();

    if schema_present {
        println!("Schema already up to date (genshin_map)");
    } else {
        let schema = Schema::new(DbBackend::Postgres);
        // Order matters for the FOREIGN KEY constraints: referenced tables
        // must exist first. sys_user → standalone masters (area is
        // self-referencing via parent_id, which is fine) → link tables →
        // system aux tables.
        let created = ensure_tables!(
            db,
            schema,
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
        );
        println!("Schema ready: {created} tables ensured in genshin_map");
    }

    // Performance indexes (idempotent). Always runs — also on the on-demand
    // path, where tables already exist but the indexes may not.
    ensure_indexes(&db).await?;

    ensure_admin_account(&db).await?;
    Ok(())
}

/// Apply the performance indexes from `scripts/indexes_dev.sql`
/// (`CREATE INDEX IF NOT EXISTS`, idempotent). The same file is executed
/// manually on the production database by ops; see its header comment.
async fn ensure_indexes(db: &sea_orm::DatabaseConnection) -> Result<()> {
    // Strip full-line comments, then execute statement by statement.
    let sql = r#"-- scripts/indexes_dev.sql
-- Performance indexes for the `genshin_map` schema (PostgreSQL 15).
--
-- Index gaps identified in the db_audit.md audit (P2): history (460K rows)
-- filters by creator_id / edit_type and defaults to ORDER BY update_time DESC
-- with no backing index; sys_user_device / sys_user_invitation /
-- sys_action_log / marker_item_link (707K rows) are filtered on unindexed
-- columns.
--
-- Idempotent: every statement uses CREATE INDEX IF NOT EXISTS, so this file
-- can be re-run safely at any time.
--
-- Where it runs:
--   1. local / e2e databases: applied automatically by `cargo run --bin
--      init_db` (init_db.rs embeds this file via include_str!).
--   2. production database: run once manually by ops, e.g.:
--        psql "postgres://<user>:<pass>@<host>:<port>/genshin_map" \
--          -f scripts/indexes_dev.sql
--      (init_db is never pointed at the production DB; the CREATE TABLE pass
--      would be skipped there anyway, and the indexes below are exactly what
--      production needs.)

-- history: per-creator filters, per-edit_type filters, and the default
-- ORDER BY update_time DESC used by history.rs list queries.
CREATE INDEX IF NOT EXISTS idx_history_creator_id ON genshin_map.history (creator_id);
CREATE INDEX IF NOT EXISTS idx_history_edit_type ON genshin_map.history (edit_type);
CREATE INDEX IF NOT EXISTS idx_history_update_time ON genshin_map.history (update_time DESC);

-- sys_user_device: login registration / access-policy checks are keyed on
-- (user_id, last_login_time).
CREATE INDEX IF NOT EXISTS idx_sys_user_device_user_last_login
    ON genshin_map.sys_user_device (user_id, last_login_time);

-- sys_user_invitation: code lookup on consume, creator_id for listing.
CREATE INDEX IF NOT EXISTS idx_sys_user_invitation_code ON genshin_map.sys_user_invitation (code);
CREATE INDEX IF NOT EXISTS idx_sys_user_invitation_creator_id
    ON genshin_map.sys_user_invitation (creator_id);

-- sys_action_log: per-user filters and create_time range scans.
CREATE INDEX IF NOT EXISTS idx_sys_action_log_user_id ON genshin_map.sys_action_log (user_id);
CREATE INDEX IF NOT EXISTS idx_sys_action_log_create_time ON genshin_map.sys_action_log (create_time);

-- marker_item_link (707K rows): standalone lookups/joins on either side.
-- (The (item_id, marker_id) composite index already exists; these two cover
-- queries that filter on one side only.)
CREATE INDEX IF NOT EXISTS idx_marker_item_link_item_id ON genshin_map.marker_item_link (item_id);
CREATE INDEX IF NOT EXISTS idx_marker_item_link_marker_id ON genshin_map.marker_item_link (marker_id);"#
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("--")
        })
        .collect::<Vec<_>>()
        .join(" ");
    for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        db.execute_unprepared(stmt)
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
        version: Set(0),
        id: sea_orm::ActiveValue::NotSet,
        create_time: Set(now),
        update_time: Set(None),
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
