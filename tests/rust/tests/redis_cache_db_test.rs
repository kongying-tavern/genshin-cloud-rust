//! Redis second-level BinaryMD5 cache test (PLAN.md M4 / F10).
//!
//! Same `GCS_TEST_DB` gate as the other DB tests; additionally requires a
//! reachable Redis (the backend's `DB_CONN.redis_conn` must be `Some` — i.e.
//! `REDIS_HOST`/`REDIS_PORT` set and the instance up, e.g. via
//! `tests/docker/docker-compose.e2e.yml`). Skips when either is missing —
//! CI's `integration` job only provisions Postgres.
//!
//! Verifies the marker-doc result flows through Redis: a compute stores a
//! versioned `binmd5:result:{epoch}:marker:result` key, `invalidate_all`
//! bumps the epoch, and the next compute stores under the new epoch.

use redis::AsyncCommands;

use _database::DB_CONN;
use _functions::functions::api::{binary_doc, marker_doc};
use _utils::{
    jwt::AuthInfo,
    models::SysUserVO,
    types::{AccessPolicyList, SystemUserRole},
};
use sea_orm::{ConnectionTrait, Schema, sea_query::TableCreateStatement};

/// Skip when Postgres+Redis are not configured (mirrors `api_db_test::db`).
async fn redis() -> Option<redis::aio::MultiplexedConnection> {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!(
            "skipped: set GCS_TEST_DB=1 with Postgres+Redis running \
             (tests/docker/docker-compose.e2e.yml) to run"
        );
        return None;
    }
    if DB_CONN.get().is_none() {
        let _ = _database::init_db_conn().await;
    }
    let conn = DB_CONN.get()?;
    let client = conn.redis_conn.as_ref()?;
    let c = client.get_multiplexed_async_connection().await.ok()?;
    Some(c)
}

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

/// Create the `marker` table (FKs stripped) so the marker-doc pipeline has
/// something to scan. Mirrors `api_db_test::ddl_without_foreign_keys`.
async fn ensure_marker_table(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    use _database::models::marker::marker as marker_model;
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let stmt: TableCreateStatement = schema.create_table_from_entity(marker_model::Entity);
    let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder);
    let stripped = regex_lite::Regex::new(
        r#",(?:\s)*CONSTRAINT "fk-[^"]+" FOREIGN KEY \([^)]+\) REFERENCES (?:"[^"]+"\.)?"[^"]+" \([^)]+\)"#,
    )
    .expect("static regex")
    .replace_all(&sql, "");
    db.execute_unprepared("DROP TABLE IF EXISTS marker CASCADE")
        .await?;
    db.execute_unprepared(&stripped).await?;
    Ok(())
}

#[tokio::test]
async fn marker_result_flows_through_redis_and_invalidate_bumps_epoch() {
    let Some(mut r) = redis().await else { return };
    let db = &DB_CONN.wait().pg_conn;
    ensure_marker_table(db).await.expect("create marker table");

    let auth = stub_auth();

    // 1) First compute stores a versioned key under epoch 1.
    let _: Result<i64, redis::RedisError> = r.del("binmd5:epoch").await;
    let entries1 = marker_doc::do_list_page_bin_md5(auth.clone(), serde_json::Value::Null)
        .await
        .expect("first marker md5 list");
    let epoch1: i64 = r.get("binmd5:epoch").await.expect("epoch key exists");
    assert_eq!(epoch1, 1);
    let raw1: Option<String> = r.get("binmd5:result:1:marker:result").await.expect("get");
    assert!(raw1.is_some(), "epoch-1 result must be stored in Redis");
    let stored: serde_json::Value = serde_json::from_str(raw1.as_deref().unwrap()).unwrap();
    let expected_len = entries1.data.as_ref().map_or(0, Vec::len);
    assert_eq!(stored.as_array().map_or(0, Vec::len), expected_len);

    // 2) invalidate_all bumps the epoch; the old key is no longer consulted.
    binary_doc::invalidate_all().await;
    let epoch2: i64 = r.get("binmd5:epoch").await.expect("epoch bumped");
    assert_eq!(epoch2, 2, "invalidate_all must bump the Redis epoch");

    // 3) The next request computes + stores under the new epoch.
    let _ = marker_doc::do_list_page_bin_md5(auth, serde_json::Value::Null)
        .await
        .expect("second marker md5 list after invalidate");
    let raw2: Option<String> = r.get("binmd5:result:2:marker:result").await.expect("get");
    assert!(
        raw2.is_some(),
        "epoch-2 result must be stored after invalidate"
    );
}
