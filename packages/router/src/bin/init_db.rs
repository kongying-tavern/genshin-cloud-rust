//! Idempotent schema initializer for local/e2e development.
//!
//! Creates the `genshin_map` schema and every table from the sea-orm models
//! (`CREATE TABLE IF NOT EXISTS`), then exits. DDL is generated from the
//! entity definitions, so it can never drift from the code.
//!
//! Usage (from the workspace root):
//!   cargo run --bin init_db
//!
//! DB connection comes from the standard `DB_*` env vars (see `.env.example`);
//! `scripts/init_db.py` wraps this for the e2e/dev workflow.

use anyhow::{Context, Result};

use sea_orm::{Database, DbBackend, Schema, prelude::*};

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

    let schema = Schema::new(DbBackend::Postgres);
    // Order matters for the FOREIGN KEY constraints: referenced tables must
    // exist first. sys_user → standalone masters (area is self-referencing
    // via parent_id, which is fine) → link tables → system aux tables.
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
    Ok(())
}
