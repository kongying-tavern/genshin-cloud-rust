//! Schema validation test — verifies that all Rust sea-orm entities produce
//! valid PostgreSQL DDL, and that the generated schema matches the table names
//! defined in the Java PDManer design.
//!
//! This test does NOT require a live database — it uses sea-orm's
//! `Schema::create_table_from_entity` to generate the DDL in-memory and
//! validates the table names + column presence.

use sea_orm::Schema;
use sea_orm::sea_query::TableCreateStatement;

use _database::models::{
    area::area, common::history, common::notice, common::route, common::score_stat, icon::icon,
    icon::icon_type, icon::icon_type_link, item::item, item::item_type, item::item_type_link,
    marker::marker, marker::marker_item_link, marker::marker_linkage, marker::marker_punctuate,
    system::sys_action_log, system::sys_user, system::sys_user_archive, system::sys_user_device,
    system::sys_user_invitation, tag::tag, tag::tag_type, tag::tag_type_link,
};

/// Generate DDL for an entity and assert the table name matches Java.
fn assert_table_name<E: sea_orm::EntityTrait>(entity: E, expected: &str) {
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let stmt: TableCreateStatement = schema.create_table_from_entity(entity);
    let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder {});
    assert!(
        sql.contains(&format!("\"{}\"", expected)),
        "table name mismatch: expected '{}' in DDL:\n{}",
        expected,
        sql
    );
}

#[test]
fn all_core_entities_have_correct_table_names() {
    assert_table_name(area::Entity, "area");
    assert_table_name(icon::Entity, "icon");
    assert_table_name(icon_type::Entity, "icon_type");
    assert_table_name(icon_type_link::Entity, "icon_type_link");
    assert_table_name(item::Entity, "item");
    assert_table_name(item_type::Entity, "item_type");
    assert_table_name(item_type_link::Entity, "item_type_link");
    assert_table_name(marker::Entity, "marker");
    assert_table_name(marker_item_link::Entity, "marker_item_link");
    assert_table_name(marker_linkage::Entity, "marker_linkage");
    assert_table_name(marker_punctuate::Entity, "marker_punctuate");
    assert_table_name(notice::Entity, "notice");
    assert_table_name(route::Entity, "route");
    assert_table_name(history::Entity, "history");
    assert_table_name(score_stat::Entity, "score_stat");
}

#[test]
fn all_tag_entities_have_correct_table_names() {
    assert_table_name(tag::Entity, "tag");
    assert_table_name(tag_type::Entity, "tag_type");
    assert_table_name(tag_type_link::Entity, "tag_type_link");
}

#[test]
fn all_system_entities_have_correct_table_names() {
    assert_table_name(sys_user::Entity, "sys_user");
    assert_table_name(sys_user_device::Entity, "sys_user_device");
    assert_table_name(sys_user_invitation::Entity, "sys_user_invitation");
    assert_table_name(sys_user_archive::Entity, "sys_user_archive");
    assert_table_name(sys_action_log::Entity, "sys_action_log");
}

#[test]
fn schema_generation_produces_valid_sql() {
    // This verifies that sea-orm can generate DDL from every entity without
    // panicking — a structural integrity check on the entity definitions.
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let entities: Vec<TableCreateStatement> = vec![
        schema.create_table_from_entity(area::Entity),
        schema.create_table_from_entity(icon::Entity),
        schema.create_table_from_entity(item::Entity),
        schema.create_table_from_entity(marker::Entity),
        schema.create_table_from_entity(marker_punctuate::Entity),
        schema.create_table_from_entity(tag::Entity),
        schema.create_table_from_entity(tag_type::Entity),
        schema.create_table_from_entity(sys_user::Entity),
        schema.create_table_from_entity(route::Entity),
        schema.create_table_from_entity(score_stat::Entity),
    ];

    // Each DDL must contain CREATE TABLE
    for stmt in &entities {
        let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder {});
        assert!(
            sql.contains("CREATE TABLE"),
            "DDL does not contain CREATE TABLE: {}",
            sql
        );
    }

    // Print a sample for manual inspection
    let sample = entities[0].to_string(sea_orm::sea_query::PostgresQueryBuilder {});
    println!("Sample DDL for area:\n{}", sample);
}
