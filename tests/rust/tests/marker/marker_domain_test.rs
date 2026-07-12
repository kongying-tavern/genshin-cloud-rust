//! Marker-domain smoke tests.
//!
//! Mirrors `area_domain_test.rs` for the marker entity. Marker is the most
//! query-heavy domain (the `*_doc` binary-archive endpoints are built around
//! it), so it gets its own test file early as the second reference port.

use sea_orm::{EntityName, Iden, Iterable};

use _database::models::marker::marker::{Column, Entity};

#[test]
fn marker_entity_table_name_matches_java() {
    assert_eq!(Entity.table_name(), "marker");
}

#[test]
fn marker_entity_has_hidden_flag_column() {
    // Marker queries filter by hiddenFlagList (normal vs insider data level).
    // This column drives the cache-splitter logic ported from Java's
    // HiddenFlagEnum + CacheSplitterEnum.
    let has_hidden_flag = Column::iter()
        .map(|c| c.to_string().to_lowercase())
        .any(|s| s.contains("hidden_flag"));
    assert!(
        has_hidden_flag,
        "marker entity must expose a hidden_flag column (Java parity)"
    );
}

#[test]
fn marker_entity_has_version_column() {
    let has_version = Column::iter()
        .map(|c| c.to_string().to_lowercase())
        .any(|s| s == "version");
    assert!(
        has_version,
        "marker entity must expose a version column (optimistic lock)"
    );
}

#[test]
fn marker_entity_has_del_flag_column() {
    // Soft delete — SafeEntityTrait::find_safety filters del_flag = false.
    let has_del_flag = Column::iter()
        .map(|c| c.to_string().to_lowercase())
        .any(|s| s.contains("del_flag"));
    assert!(
        has_del_flag,
        "marker entity must expose a del_flag column (soft delete)"
    );
}
