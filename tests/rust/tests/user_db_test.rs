//! DB-backed integration test for the `sys_user` domain — the template for all
//! future domain DB tests (M1, PLAN.md §4).
//!
//! Gated on the `GCS_TEST_DB` env var: the CI `integration` job sets it and
//! provisions a Postgres service; everywhere else this test skips cleanly so
//! `cargo test --workspace` stays green without a database.
//!
//! What it verifies (the full DB harness, not just schema):
//!   1. `init_db_conn()` connects to Postgres (Redis/MinIO degrade gracefully).
//!   2. `Schema::create_table_from_entity` builds the `sys_user` table on a live
//!      database (so entity definitions are executable DDL, not just string
//!      matches — a stronger guarantee than `schema_test.rs`).
//!   3. The `SafeEntityTrait` round-trip: insert → `find_safety_by_id` read →
//!      `delete_safety` soft-delete → read again returns `None`.
//!
//! Future domain ports (icon, item, tag, ...) should copy this file, swap the
//! entity, and extend the assertions — the harness setup is reusable as-is.

use sea_orm::{
    ActiveValue::NotSet, ConnectionTrait, EntityTrait, Schema, sea_query::TableCreateStatement,
};

use _database::DB_CONN;
use _database::models::system::sys_user;
use _functions::functions::system::user as user_fns;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::SysUserVO,
    types::{AccessPolicyItemEnum, AccessPolicyList, SystemUserRole},
};

/// Skip the test when no database is configured. Returns the shared connection
/// on success, or prints a skip notice and returns `None`.
async fn db() -> Option<&'static sea_orm::DatabaseConnection> {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!("skipped: set GCS_TEST_DB=1 with a reachable Postgres to run");
        return None;
    }
    if DB_CONN.get().is_none() {
        // init_db_conn is idempotent in practice: if another test already set
        // the global, this returns Err and we fall through to the get() below.
        let _ = _database::init_db_conn().await;
    }
    DB_CONN.get().map(|m| &m.pg_conn)
}

/// Recreate the `sys_user` table in the `genshin_map` schema. Idempotent so the
/// test can run repeatedly. `sys_user` only self-references (creator/updater),
/// so it can be created in isolation — no foreign-key ordering concerns.
async fn recreate_table(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS genshin_map")
        .await?;
    db.execute_unprepared(r#"DROP TABLE IF EXISTS "genshin_map"."sys_user" CASCADE"#)
        .await?;

    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let stmt: TableCreateStatement = schema.create_table_from_entity(sys_user::Entity);
    let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder);
    db.execute_unprepared(&sql).await?;
    Ok(())
}

/// Insert a user row directly via the entity (avoids the bcrypt round-trip of
/// `do_register`). Lets the IDENTITY column assign the id and returns it.
async fn seed_user(db: &sea_orm::DatabaseConnection) -> anyhow::Result<i64> {
    use chrono::Utc;
    use sea_orm::ActiveValue::Set;

    let am = sys_user::ActiveModel {
        id: NotSet,
        version: Set(0),
        create_time: Set(Utc::now().naive_utc()),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        username: Set("db_test_user".into()),
        password: Set(_utils::bcrypt::generate_storage_password("init_pw").unwrap()),
        nickname: Set(Some("DB Test".into())),
        qq: Set(None),
        phone: Set(None),
        logo: Set(None),
        role_id: Set(SystemUserRole::MapUser),
        access_policy: Set(Some(AccessPolicyList(vec![
            AccessPolicyItemEnum::IpSameLastIp,
        ]))),
        remark: Set(None),
    };
    let res = sys_user::Entity::insert(am).exec(db).await?;
    Ok(res.last_insert_id)
}

/// A minimal `AuthInfo` for calling business functions that take it (currently
/// unused by the assertions below, but part of the template for future tests).
#[allow(dead_code)]
fn stub_auth() -> AuthInfo {
    let now = chrono::Utc::now();
    AuthInfo {
        info: SysUserVO {
            id: 0,
            username: "stub".into(),
            nickname: None,
            qq: None,
            phone: None,
            logo: None,
            role_id: SystemUserRole::Admin,
            access_policy: AccessPolicyList(vec![]),
            remark: None,
        },
        created_at: now,
        expires_at: now + chrono::Duration::days(1),
    }
}

#[tokio::test]
async fn user_db_round_trip() {
    // JWT_SECRET is required (no default); set a test secret before any JWT
    // operation touches the lazy key material.
    // SAFETY: single-threaded test process; set once at the start.
    unsafe {
        std::env::set_var("JWT_SECRET", "integration-test-secret");
    }

    let Some(db) = db().await else {
        return;
    };

    // ── Setup: rebuild the table ────────────────────────────────────────────
    recreate_table(db).await.expect("recreate sys_user table");

    // ── Seed ─────────────────────────────────────────────────────────────────
    let id = seed_user(db).await.expect("seed user");
    assert!(id != 0, "IDENTITY column should assign a non-zero id");

    // ── Read via the business layer (do_get_info → find_safety_by_id) ────────
    let vo = user_fns::do_get_info(stub_auth(), id)
        .await
        .expect("do_get_info reads the seeded user");
    assert_eq!(vo.username, "db_test_user");
    assert_eq!(vo.role_id, SystemUserRole::MapUser);

    // ── do_update_password: wrong old password must fail ──────────────────────
    let wrong_pw = user_fns::do_update_password(
        stub_auth(),
        vec![],
        id,
        String::new(),
        "wrong_old_pw".into(),
        String::new(),
        SystemUserRole::MapUser,
        "new_pw".into(),
    )
    .await;
    assert!(
        wrong_pw.is_err(),
        "updating the password with a wrong old password must fail"
    );

    // ── do_update_password: correct old password succeeds and the new
    //    password verifies ────────────────────────────────────────────────────
    user_fns::do_update_password(
        stub_auth(),
        vec![],
        id,
        String::new(),
        "init_pw".into(),
        String::new(),
        SystemUserRole::MapUser,
        "new_pw".into(),
    )
    .await
    .expect("update password with correct old password");

    let updated = sys_user::Entity::find_safety_by_id(id)
        .one(db)
        .await
        .expect("fetch updated user")
        .expect("user still exists");
    assert!(
        _utils::bcrypt::verify_password("new_pw", updated.password.clone()).expect("verify"),
        "the new password must verify after the update"
    );
    assert!(
        !_utils::bcrypt::verify_password("init_pw", updated.password).expect("verify"),
        "the old password must no longer verify"
    );

    // ── do_kick_out: no-op degradation without Redis (must not error) ────────
    user_fns::do_kick_out(stub_auth(), id.to_string())
        .await
        .expect("kick_out degrades gracefully without Redis");

    // ── do_delete (soft delete via delete_safety_by_id) ──────────────────────
    user_fns::do_delete(stub_auth(), id)
        .await
        .expect("do_delete soft-deletes the user");

    let gone = sys_user::Entity::find_safety_by_id(id)
        .one(db)
        .await
        .expect("read after soft-delete");
    assert!(
        gone.is_none(),
        "soft-deleted row must be filtered out by find_safety"
    );
}
