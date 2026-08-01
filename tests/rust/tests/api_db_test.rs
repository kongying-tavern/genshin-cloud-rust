//! DB-backed business-assertion test for the area + item_doc domains (M1 third
//! item, PLAN.md §4).
//!
//! Same `GCS_TEST_DB` gate as `user_db_test`. Upgrades the e2e smoke checks
//! (which treated 401/403 as "route exists ✓") into real data assertions:
//! seeds rows and verifies the business-layer functions return them. Exercises
//! the `SafeEntityTrait` query path and the BinaryMD5 pipeline end-to-end.
//!
//! Tables are created with foreign-key constraints stripped (regex on the
//! generated DDL) so each table is independent — no need to build the full
//! dependency graph (sys_user → icon → icon_type → area → item → ...) just to
//! test one domain. Future domain tests can reuse `recreate_tables_fklless`.

use sea_orm::{
    ActiveValue::NotSet, ConnectionTrait, EntityTrait, Schema, sea_query::TableCreateStatement,
};

use _database::DB_CONN;
use _database::models::{area::area as area_model, item::item as item_model};
use _functions::functions::api::{area as area_fns, item_doc};
use _utils::{
    jwt::AuthInfo,
    models::{
        SysUserVO,
        area::{AreaAddRequest, AreaListRequest},
    },
    types::{AccessPolicyList, HiddenFlag, IconStyleType, SystemUserRole},
};

/// Skip when no database is configured. Mirrors `user_db_test::db`.
async fn db() -> Option<&'static sea_orm::DatabaseConnection> {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!("skipped: set GCS_TEST_DB=1 with a reachable Postgres to run");
        return None;
    }
    if DB_CONN.get().is_none() {
        let _ = _database::init_db_conn().await;
    }
    DB_CONN.get().map(|m| &m.pg_conn)
}

/// Build a CREATE TABLE statement for an entity with all FOREIGN KEY
/// constraints stripped, so the table can be created in isolation without its
/// dependency tables existing.
fn ddl_without_foreign_keys<E>(entity: E) -> String
where
    E: sea_orm::EntityTrait,
{
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let stmt: TableCreateStatement = schema.create_table_from_entity(entity);
    let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder);
    // Remove `, CONSTRAINT "fk-..." FOREIGN KEY (...) REFERENCES "schema"."t" (...)`
    let stripped = regex_lite::Regex::new(
        r#",(?:\s)*CONSTRAINT "fk-[^"]+" FOREIGN KEY \([^)]+\) REFERENCES (?:"[^"]+"\.)?"[^"]+" \([^)]+\)"#,
    )
    .expect("static regex")
    .replace_all(&sql, "");
    stripped.into_owned()
}

/// Drop (if present) and recreate the given entities' tables, FK-free, in the
/// `genshin_map` schema. Order matters only for DROP CASCADE, not for CREATE
/// (no FKs).
async fn recreate_tables_fklless(
    db: &sea_orm::DatabaseConnection,
    table_names: &[&str],
    ddls: &[String],
) -> anyhow::Result<()> {
    db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS genshin_map")
        .await?;
    for name in table_names {
        db.execute_unprepared(&format!(
            r#"DROP TABLE IF EXISTS "genshin_map"."{name}" CASCADE"#
        ))
        .await?;
    }
    for ddl in ddls {
        db.execute_unprepared(ddl).await?;
    }
    Ok(())
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

#[tokio::test]
async fn area_and_item_doc_business_assertions() {
    let Some(db) = db().await else {
        return;
    };

    // ── Setup: FK-free tables for area + item ────────────────────────────────
    let ddls = [
        ddl_without_foreign_keys(area_model::Entity),
        ddl_without_foreign_keys(item_model::Entity),
    ];
    recreate_tables_fklless(db, &["area", "item"], &ddls)
        .await
        .expect("recreate area + item tables");

    // ── Seed one area + two items (different hidden_flag → 2 MD5 groups) ─────
    use sea_orm::ActiveValue::Set;
    let now = chrono::Utc::now().naive_utc();

    let area_am = area_model::ActiveModel {
        id: NotSet,
        version: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        name: Set("Test Area".into()),
        code: Set(None),
        content: Set(None),
        icon_id: Set(0),
        parent_id: Set(-1),
        is_final: Set(true),
        hidden_flag: Set(HiddenFlag::Visible),
        sort_index: Set(0),
        special_flag: Set(0),
    };
    let seeded_area_id = area_model::Entity::insert(area_am)
        .exec(db)
        .await
        .expect("seed area")
        .last_insert_id;

    for hidden in [HiddenFlag::Visible, HiddenFlag::Hidden] {
        let item_am = item_model::ActiveModel {
            id: NotSet,
            version: Set(0),
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),
            name: Set(format!("Test Item {hidden:?}")),
            area_id: Set(seeded_area_id),
            default_refresh_time: Set(0),
            default_content: Set(None),
            default_count: Set(1),
            icon_id: Set(0),
            icon_style_type: Set(IconStyleType::Default),
            hidden_flag: Set(hidden),
            sort_index: Set(0),
            special_flag: Set(Some(0)),
        };
        item_model::Entity::insert(item_am)
            .exec(db)
            .await
            .expect("seed item");
    }

    let auth = stub_auth();

    // ── Assertion 1: area do_list returns the seeded area ────────────────────
    let list_resp = area_fns::do_list(
        auth.clone(),
        AreaListRequest {
            is_traverse: None,
            parent_id: None,
            hidden_flag: None,
        },
    )
    .await
    .expect("area do_list")
    .data
    .expect("area list payload");
    let areas = &list_resp.0;
    assert_eq!(areas.len(), 1, "exactly one seeded area");
    assert_eq!(areas[0].name, "Test Area");
    assert_eq!(areas[0].id, seeded_area_id);

    // ── Assertion 2: area do_add inserts a second area ───────────────────────
    let add_resp = area_fns::do_add(
        auth.clone(),
        AreaAddRequest {
            name: "Second Area".into(),
            code: Some("A2".into()),
            content: None,
            icon_tag: "0".into(),
            parent_id: seeded_area_id,
            is_final: false,
            hidden_flag: HiddenFlag::Hidden,
            sort_index: 1,
            special_flag: 0,
        },
    )
    .await
    .expect("area do_add")
    .data
    .expect("area add payload");
    let new_id = add_resp.id;
    assert!(new_id != seeded_area_id, "new area gets a distinct id");

    let after = area_fns::do_list(
        auth.clone(),
        AreaListRequest {
            is_traverse: None,
            parent_id: None,
            hidden_flag: None,
        },
    )
    .await
    .expect("area do_list after add")
    .data
    .expect("area list payload");
    assert_eq!(after.0.len(), 2, "two areas after add");

    // ── Assertion 3: item_doc do_list_page_bin_md5 returns one MD5 per
    //    hidden_flag group (2 items, 2 distinct flags → 2 entries), each a
    //    32-char hex MD5. ────────────────────────────────────────────────────
    let md5_resp = item_doc::do_list_page_bin_md5(auth, serde_json::Value::Null)
        .await
        .expect("item_doc do_list_page_bin_md5")
        .data
        .expect("item_doc md5 payload");
    assert_eq!(
        md5_resp.len(),
        2,
        "two hidden_flag groups → two MD5 entries"
    );
    for vo in &md5_resp {
        assert_eq!(
            vo.md5.len(),
            32,
            "MD5 must be 32 hex chars, got: {}",
            vo.md5
        );
        assert!(
            vo.md5.chars().all(|c| c.is_ascii_hexdigit()),
            "MD5 must be hex: {}",
            vo.md5
        );
    }
}
