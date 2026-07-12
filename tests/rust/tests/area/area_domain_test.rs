//! Area-domain smoke tests.
//!
//! These tests verify the `_database::models::area` entity shape matches the
//! Java `Area` reference. They are compile-time + value assertions only — no
//! database connection is needed. This is the template for future domain
//! ports (icon, item, tag, notice, ...).

use sea_orm::{EntityName, Iden, Iterable};

use _database::models::area::area::{Column, Entity};

#[test]
fn area_entity_table_name_matches_java() {
    // The Java side maps to the `area` table under the `genshin_map` schema.
    // Keeping the table name identical lets the Rust backend share the same
    // PostgreSQL database as the Java backend during the migration.
    assert_eq!(Entity.table_name(), "area");
}

#[test]
fn area_entity_has_hidden_flag_column() {
    // hiddenFlag / data-level filtering is central to the marker & item query
    // API (normal vs insider). The column must exist on every content entity.
    let has_hidden_flag = Column::iter()
        .map(|c| c.to_string().to_lowercase())
        .any(|s| s.contains("hidden_flag"));
    assert!(
        has_hidden_flag,
        "area entity must expose a hidden_flag column (Java parity)"
    );
}

#[test]
fn area_entity_has_version_column() {
    // Optimistic-locking version column — the SafeEntityTrait macro relies on
    // it. Must be present on every entity that goes through update_safety.
    let has_version = Column::iter()
        .map(|c| c.to_string().to_lowercase())
        .any(|s| s == "version");
    assert!(
        has_version,
        "area entity must expose a version column (optimistic lock)"
    );
}
