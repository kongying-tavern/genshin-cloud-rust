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

/// Recreate the `sys_user` table in the configured schema (default
/// `genshin_map`, see `DB_SCHEMA`). Idempotent so the test can run
/// repeatedly. `sys_user` only self-references (creator/updater), so it can
/// be created in isolation — no foreign-key ordering concerns.
async fn recreate_table(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    let schema = _database::default_schema();
    db.execute_unprepared(&format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#))
        .await?;
    db.execute_unprepared(&format!(
        r#"DROP TABLE IF EXISTS "{schema}"."sys_user" CASCADE"#
    ))
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
        version: Set(1),
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
            id: 1,
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
    let wrong_pw =
        user_fns::do_update_password(stub_auth(), id, "wrong_old_pw".into(), "new_pw".into()).await;
    assert!(
        wrong_pw.is_err(),
        "updating the password with a wrong old password must fail"
    );

    // ── do_update_password: correct old password succeeds and the new
    //    password verifies ────────────────────────────────────────────────────
    user_fns::do_update_password(stub_auth(), id, "init_pw".into(), "new_pw".into())
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

    // ── Archive slot-row model (Java SysUserArchiveService) ─────────────────
    // 归档行以软删用户为属主不影响断言（实体 DDL 不建外键）。
    use _database::models::system::sys_user_archive as archive_model;
    use _functions::functions::system::archive as archive_fns;
    use sea_orm::{ColumnTrait, QueryFilter};

    recreate_archive_table(db)
        .await
        .expect("recreate sys_user_archive");
    let auth = stub_auth();

    // 首次保存：新建槽位行，data = [entry]，返回 true
    let body = serde_json::json!({"Data_KYJG": 1});
    let saved = archive_fns::do_save(auth.clone(), id, 0, None, body.clone())
        .await
        .expect("first save")
        .data
        .expect("bool payload");
    assert_eq!(saved, serde_json::json!(true));
    let rows = archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(id))
        .all(db)
        .await
        .expect("fetch rows");
    assert_eq!(rows.len(), 1, "one row per user+slot");
    let data = rows[0].data.as_array().expect("data is an array");
    assert_eq!(data.len(), 1);
    assert_eq!(
        data[0]["archive"],
        serde_json::json!({"Data_KYJG":1}).to_string()
    );
    assert!(
        data[0]["time"].as_i64().unwrap_or(0) > 0,
        "entry time is a ms number"
    );

    // 审计字段：新增时 create/update 两组全部设置
    assert_eq!(rows[0].creator_id, Some(id));
    assert_eq!(rows[0].updater_id, Some(id));
    assert!(rows[0].update_time.is_some(), "update_time set on insert");

    // 幂等：与最新一条一致 → false，不新增条目
    let dup = archive_fns::do_save(auth.clone(), id, 0, None, body.clone())
        .await
        .expect("dup save")
        .data
        .expect("bool payload");
    assert_eq!(dup, serde_json::json!(false));

    // 再存 5 条不同内容：头插 + 上限 5（共 6 条不同 → 挤掉最旧 1 条）
    for i in 2..=6 {
        archive_fns::do_save(
            auth.clone(),
            id,
            0,
            None,
            serde_json::json!({"Data_KYJG": i}),
        )
        .await
        .expect("save variant");
    }
    // 最新在前
    let last = archive_fns::do_get_last(auth.clone(), id, 0)
        .await
        .expect("get last")
        .data
        .expect("payload")
        .expect("latest entry");
    assert_eq!(last.history_index, 1);
    assert_eq!(
        last.archive,
        serde_json::json!({"Data_KYJG": 6}).to_string()
    );

    let history = archive_fns::do_get_history(auth.clone(), id, 0)
        .await
        .expect("get history")
        .data
        .expect("slot vo");
    assert_eq!(history.archive.len(), 5, "history capped at 5");
    assert_eq!(
        history.archive[0].archive,
        serde_json::json!({"Data_KYJG": 6}).to_string(),
        "newest first"
    );
    assert_eq!(
        history.archive[4].archive,
        serde_json::json!({"Data_KYJG": 2}).to_string()
    );
    for (i, vo) in history.archive.iter().enumerate() {
        assert_eq!(vo.history_index, (i + 1) as i64, "historyIndex is 1-based");
    }

    // 脏数据兼容：同 user+slot 再插一行、update_time 更新 → 取值命中最新行
    archive_model::Entity::find_safety()
        .filter(archive_model::Column::UserId.eq(id))
        .one(db)
        .await
        .expect("fetch original row")
        .expect("row exists");
    let loser = archive_model::ActiveModel {
        id: NotSet,
        version: sea_orm::ActiveValue::Set(0),
        create_time: sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc()),
        update_time: sea_orm::ActiveValue::Set(Some(
            chrono::Utc::now().naive_utc() + chrono::Duration::seconds(60),
        )),
        creator_id: sea_orm::ActiveValue::Set(Some(id)),
        updater_id: sea_orm::ActiveValue::Set(Some(id)),
        del_flag: sea_orm::ActiveValue::Set(false),
        name: sea_orm::ActiveValue::Set(None),
        slot_index: sea_orm::ActiveValue::Set(0),
        user_id: sea_orm::ActiveValue::Set(id),
        data: sea_orm::ActiveValue::Set(serde_json::json!([{
            "archive": "\"dirty winner\"",
            "time": 1,
        }])),
    };
    archive_model::Entity::insert(loser)
        .exec(db)
        .await
        .expect("seed dirty duplicate row");
    let dirty_last = archive_fns::do_get_last(auth.clone(), id, 0)
        .await
        .expect("get last with dirty rows")
        .data
        .expect("payload")
        .expect("entry from winner row");
    assert_eq!(dirty_last.archive, "\"dirty winner\"");

    // 恢复：弹出最新一条并返回（historyIndex=1）；胜出的脏行仅 1 条，
    // 弹出后 data=[]（其 update_time 被刷新，继续作为取值命中行）
    let removed = archive_fns::do_restore_slot(auth.clone(), id, 0)
        .await
        .expect("restore")
        .data
        .expect("removed entry");
    assert_eq!(removed.archive, "\"dirty winner\"");
    assert_eq!(removed.history_index, 1);
    let after = archive_fns::do_get_history(auth.clone(), id, 0)
        .await
        .expect("history after restore")
        .data
        .expect("slot vo after restore");
    assert_eq!(after.archive.len(), 0, "winner row popped to zero entries");

    // 重命名 + 删除槽位（RBoolean）；缺失槽位按 Java 文案报错
    assert!(
        archive_fns::do_rename_by_slot(auth.clone(), id, 1, "新档名".into())
            .await
            .is_err(),
        "rename a missing slot errors (槽位不存在)"
    );
    archive_fns::do_save(
        auth.clone(),
        id,
        1,
        Some("存档一".into()),
        serde_json::json!({}),
    )
    .await
    .expect("save slot 1 with name");
    let renamed = archive_fns::do_rename_by_slot(auth.clone(), id, 1, "新档名".into())
        .await
        .expect("rename")
        .data
        .expect("bool payload");
    assert!(renamed, "rename returns RBoolean true");
    let deleted = archive_fns::do_delete_slot(auth.clone(), id, 1)
        .await
        .expect("delete slot")
        .data
        .expect("bool payload");
    assert!(deleted, "delete returns RBoolean true");
    let missing = archive_fns::do_get_history(auth.clone(), id, 1).await;
    assert!(missing.is_err(), "deleted slot reports 槽位不存在");
}

/// Recreate the `sys_user_archive` table (self-contained, no FK ordering).
async fn recreate_archive_table(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    use _database::models::system::sys_user_archive;
    let schema = _database::default_schema();
    db.execute_unprepared(&format!(
        r#"DROP TABLE IF EXISTS "{schema}"."sys_user_archive" CASCADE"#
    ))
    .await?;
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let stmt: TableCreateStatement = schema.create_table_from_entity(sys_user_archive::Entity);
    db.execute_unprepared(&stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder))
        .await?;
    Ok(())
}
