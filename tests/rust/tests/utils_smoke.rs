//! Smoke tests for `_utils` — verify the public surface compiles and the
//! core value types round-trip correctly. These are the first rung of the
//! test ladder; domain integration tests live under `tests/area/` and
//! `tests/marker/`.

use _utils::types::HiddenFlag;

#[test]
fn hidden_flag_visible_is_zero() {
    // The Java side encodes data-level visibility as a numeric flag; 0 means
    // visible to everyone. This invariant must hold for the Rust port.
    assert_eq!(HiddenFlag::Visible as i32, 0);
}

#[test]
fn hidden_flag_hidden_is_one() {
    assert_eq!(HiddenFlag::Hidden as i32, 1);
}

#[test]
fn hidden_flag_variants_are_distinct() {
    // Every variant must map to a unique discriminant so that filtering by
    // hiddenFlagList works the same as the Java `HiddenFlagEnum`.
    let values = [HiddenFlag::Visible as i32, HiddenFlag::Hidden as i32];
    let unique: std::collections::HashSet<i32> = values.into_iter().collect();
    assert_eq!(unique.len(), values.len(), "HiddenFlag variants collide");
}
