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

use _database::DB_CONN;
use _database::models::{
    area::area as area_model, common::history as history_model,
    common::score_stat as score_stat_model, item::item as item_model,
    marker::marker as marker_model, marker::marker_item_link as mil_model,
    marker::marker_punctuate as mp_model, system::sys_action_log as action_log_model,
    system::sys_user as sys_user_model, system::sys_user_device as device_model,
};
use _functions::functions::api::{
    area as area_fns, cache as cache_fns, item_doc, marker as marker_fns,
    punctuate_audit as audit_fns, score as score_fns,
};
use _functions::functions::system::oauth as oauth_fns;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        SysUserVO,
        area::{AreaAddRequest, AreaListRequest},
        marker::{
            MarkerTweakConfig, MarkerTweakConfigPropEnum, MarkerTweakConfigTypeEnum,
            MarkerTweakRequest, TweakMeta,
        },
        score::{ScoreDataRequest, ScoreGenerateRequest},
    },
    types::{
        AccessPolicyItemEnum, AccessPolicyList, HiddenFlag, HistoryEditType, HistoryOperationType,
        IconStyleType, MarkerPunctuateMethodType, MarkerPunctuateStatus, SystemUserRole,
    },
};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Schema,
    sea_query::TableCreateStatement,
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

async fn link_count(db: &sea_orm::DatabaseConnection, marker_id: i64) -> anyhow::Result<u64> {
    Ok(mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.eq(marker_id))
        .count(db)
        .await?)
}

async fn seed_punctuate(
    db: &sea_orm::DatabaseConnection,
    punctuate_id: i64,
    method_type: MarkerPunctuateMethodType,
    now: chrono::NaiveDateTime,
) -> anyhow::Result<i64> {
    let am = mp_model::ActiveModel {
        id: NotSet,
        version: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        punctuate_id: Set(punctuate_id),
        original_marker_id: Set(None),
        marker_title: Set(Some("Audit Marker".into())),
        item_list: Set(serde_json::json!([])),
        position: Set("3.0,4.0".into()),
        content: Set(String::new()),
        picture: Set(None),
        marker_creator_id: Set(1),
        picture_creator_id: Set(None),
        video_path: Set(None),
        author: Set(2),
        status: Set(MarkerPunctuateStatus::Reviewing),
        audit_remark: Set(None),
        method_type: Set(method_type),
        refresh_time: Set(0),
        hidden_flag: Set(HiddenFlag::Visible),
        extra: Set(None),
    };
    Ok(mp_model::Entity::insert(am).exec(db).await?.last_insert_id)
}

async fn seed_user(
    db: &sea_orm::DatabaseConnection,
    username: &str,
    policy: Vec<AccessPolicyItemEnum>,
    qq: Option<String>,
    now: chrono::NaiveDateTime,
) -> anyhow::Result<i64> {
    let am = sys_user_model::ActiveModel {
        id: NotSet,
        version: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        username: Set(username.to_string()),
        password: Set(_utils::bcrypt::generate_storage_password("pw123").unwrap()),
        nickname: Set(None),
        qq: Set(qq),
        phone: Set(None),
        logo: Set(None),
        role_id: Set(SystemUserRole::MapUser),
        access_policy: Set(Some(AccessPolicyList(policy))),
        remark: Set(None),
    };
    Ok(sys_user_model::Entity::insert(am)
        .exec(db)
        .await?
        .last_insert_id)
}

fn stub_auth() -> AuthInfo {
    stub_auth_with_role(SystemUserRole::Admin)
}

fn stub_auth_with_role(role: SystemUserRole) -> AuthInfo {
    let now = chrono::Utc::now();
    AuthInfo {
        info: SysUserVO {
            id: 1,
            username: "stub".into(),
            nickname: None,
            qq: None,
            phone: None,
            logo: None,
            role_id: role,
            access_policy: AccessPolicyList(vec![]),
            remark: None,
        },
        created_at: now,
        expires_at: now + chrono::Duration::days(1),
    }
}

/// The anonymous client-credentials identity (user id 0): read-only.
fn stub_anonymous_auth() -> AuthInfo {
    let mut auth = stub_auth_with_role(SystemUserRole::Visitor);
    auth.info.id = 0;
    auth
}

#[tokio::test]
async fn area_and_item_doc_business_assertions() {
    // Enable RS256 signing for the whole test process: generate an ephemeral
    // RSA key and set JWT_RSA_PRIVATE_KEY_PEM BEFORE any JWT operation touches
    // the lazy key material. Every login/token flow below then exercises the
    // RS256 sign/verify path end-to-end (and the JWKS assertions check the
    // RSA key shape).
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};
    let rsa_key =
        rsa::RsaPrivateKey::new(&mut rand_core::OsRng, 2048).expect("generate ephemeral RSA key");
    let rsa_pem = rsa_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("encode RSA private key")
        .to_string();
    // SAFETY: single-threaded test process; the env var is set once before any
    // JWT operation reads the lazy key material (edition 2024 marks set_var
    // unsafe because concurrent readers could race).
    unsafe {
        std::env::set_var("JWT_RSA_PRIVATE_KEY_PEM", rsa_pem);
        std::env::set_var("JWT_SECRET", "integration-test-secret");
    }

    let Some(db) = db().await else {
        return;
    };

    // ── Setup: FK-free tables for area + item ────────────────────────────────
    let ddls = [
        ddl_without_foreign_keys(sys_user_model::Entity),
        ddl_without_foreign_keys(device_model::Entity),
        ddl_without_foreign_keys(action_log_model::Entity),
        ddl_without_foreign_keys(history_model::Entity),
        ddl_without_foreign_keys(score_stat_model::Entity),
        ddl_without_foreign_keys(area_model::Entity),
        ddl_without_foreign_keys(item_model::Entity),
        ddl_without_foreign_keys(marker_model::Entity),
        ddl_without_foreign_keys(mil_model::Entity),
        ddl_without_foreign_keys(mp_model::Entity),
    ];
    recreate_tables_fklless(
        db,
        &[
            "sys_user",
            "sys_user_device",
            "sys_action_log",
            "history",
            "score_stat",
            "area",
            "item",
            "marker",
            "marker_item_link",
            "marker_punctuate",
        ],
        &ddls,
    )
    .await
    .expect("recreate area + item + marker + punctuate tables");

    // ── Seed one area + two items (different hidden_flag → 2 MD5 groups) ─────
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
        icon_tag: Set("0".into()),
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
            icon_tag: Set("0".into()),
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

    // A second business-layer insert must not collide on the identity column
    // (regression for the id: Set(0) bug — identity columns must be NotSet).
    area_fns::do_add(
        auth.clone(),
        AreaAddRequest {
            name: "Third Area".into(),
            code: None,
            content: None,
            icon_tag: "0".into(),
            parent_id: seeded_area_id,
            is_final: false,
            hidden_flag: HiddenFlag::Visible,
            sort_index: 2,
            special_flag: 0,
        },
    )
    .await
    .expect("second do_add must not collide on the identity column")
    .data
    .expect("area add payload");
    let after_third = area_fns::do_list(
        auth.clone(),
        AreaListRequest {
            is_traverse: None,
            parent_id: None,
            hidden_flag: None,
        },
    )
    .await
    .expect("area do_list after third add")
    .data
    .expect("area list payload");
    assert_eq!(after_third.0.len(), 3, "three areas after repeated adds");

    // Anonymous (client-credentials, id=0) tokens must be rejected on writes.
    assert!(
        area_fns::do_add(
            stub_anonymous_auth(),
            AreaAddRequest {
                name: "Anonymous Rejected".into(),
                code: None,
                content: None,
                icon_tag: "0".into(),
                parent_id: -1,
                is_final: false,
                hidden_flag: HiddenFlag::Visible,
                sort_index: 0,
                special_flag: 0,
            },
        )
        .await
        .is_err(),
        "anonymous token must be rejected for area writes"
    );

    // ── Assertion 3: item_doc do_list_page_bin_md5 returns one MD5 per
    //    hidden_flag group (2 items, 2 distinct flags → 2 entries), each a
    //    32-char hex MD5. ────────────────────────────────────────────────────
    let md5_resp = item_doc::do_list_page_bin_md5(auth.clone(), serde_json::Value::Null)
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

    // The BinaryMD5 pages are cached in-process: a second call must return the
    // same md5 AND the same stable `time` (not a fresh request timestamp).
    let md5_resp_2 = item_doc::do_list_page_bin_md5(auth.clone(), serde_json::Value::Null)
        .await
        .expect("item_doc md5 second call")
        .data
        .expect("item_doc md5 payload");
    assert_eq!(
        md5_resp_2.len(),
        md5_resp.len(),
        "same page count on second call"
    );
    for (first, second) in md5_resp.iter().zip(md5_resp_2.iter()) {
        assert_eq!(first.md5, second.md5, "md5 must be stable across calls");
        assert_eq!(
            first.time, second.time,
            "time must be stable while the page is cached"
        );
    }

    // The refresh endpoint must invalidate the cache: the next md5 list is
    // regenerated (fresh time), proving the flush is wired.
    cache_fns::do_delete_item_cache(auth.clone())
        .await
        .expect("delete item cache")
        .data
        .expect("cache delete ok");
    let md5_resp_3 = item_doc::do_list_page_bin_md5(auth.clone(), serde_json::Value::Null)
        .await
        .expect("item_doc md5 after refresh")
        .data
        .expect("item_doc md5 payload");
    assert_eq!(
        md5_resp_3.len(),
        md5_resp.len(),
        "same page count after refresh"
    );
    for (before, after) in md5_resp_2.iter().zip(md5_resp_3.iter()) {
        assert_eq!(before.md5, after.md5, "md5 must be deterministic");
        assert_ne!(
            before.time, after.time,
            "refresh must regenerate the page (fresh time)"
        );
    }

    // ── Assertion 4: marker ItemList tweak maintains marker_item_link ────────
    // Seed one marker, then exercise Append / InsertIfAbsent / Replace /
    // RemoveLeft against its item links.
    let marker_am = marker_model::ActiveModel {
        id: NotSet,
        version: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        marker_stamp: Set(None),
        marker_title: Set(Some("Test Marker".into())),
        position: Set("1.0,2.0".into()),
        content: Set(String::new()),
        picture: Set(None),
        marker_creator_id: Set(1),
        picture_creator_id: Set(None),
        video_path: Set(None),
        refresh_time: Set(0),
        hidden_flag: Set(HiddenFlag::Visible),
        extra: Set(None),
    };
    let marker_id = marker_model::Entity::insert(marker_am)
        .exec(db)
        .await
        .expect("seed marker")
        .last_insert_id;

    // Append item 1001 + 1002
    marker_fns::do_tweak(
        auth.clone(),
        MarkerTweakRequest {
            marker_ids: vec![marker_id],
            tweaks: vec![MarkerTweakConfig {
                meta: TweakMeta {
                    item_list: Some(vec![
                        Some(serde_json::json!(1001)),
                        Some(serde_json::json!(1002)),
                    ]),
                    map: None,
                    replace: None,
                    test: None,
                    value: None,
                },
                prop: MarkerTweakConfigPropEnum::ItemList,
                marker_tweak_config_type: MarkerTweakConfigTypeEnum::Append,
            }],
        },
    )
    .await
    .expect("append item links")
    .data
    .expect("tweak ok");
    assert_eq!(
        link_count(db, marker_id).await.expect("count"),
        2,
        "append creates two links"
    );

    // InsertIfAbsent with an existing id must not duplicate
    marker_fns::do_tweak(
        auth.clone(),
        MarkerTweakRequest {
            marker_ids: vec![marker_id],
            tweaks: vec![MarkerTweakConfig {
                meta: TweakMeta {
                    item_list: Some(vec![Some(serde_json::json!(1001))]),
                    map: None,
                    replace: None,
                    test: None,
                    value: None,
                },
                prop: MarkerTweakConfigPropEnum::ItemList,
                marker_tweak_config_type: MarkerTweakConfigTypeEnum::InsertIfAbsent,
            }],
        },
    )
    .await
    .expect("insert-if-absent tweak")
    .data
    .expect("tweak ok");
    assert_eq!(
        link_count(db, marker_id).await.expect("count"),
        2,
        "InsertIfAbsent must not duplicate an existing link"
    );

    // Replace with a single item 1003
    marker_fns::do_tweak(
        auth.clone(),
        MarkerTweakRequest {
            marker_ids: vec![marker_id],
            tweaks: vec![MarkerTweakConfig {
                meta: TweakMeta {
                    item_list: Some(vec![Some(serde_json::json!(1003))]),
                    map: None,
                    replace: None,
                    test: None,
                    value: None,
                },
                prop: MarkerTweakConfigPropEnum::ItemList,
                marker_tweak_config_type: MarkerTweakConfigTypeEnum::Replace,
            }],
        },
    )
    .await
    .expect("replace tweak")
    .data
    .expect("tweak ok");
    assert_eq!(
        link_count(db, marker_id).await.expect("count"),
        1,
        "replace collapses to a single link"
    );

    // RemoveLeft removes 1003
    marker_fns::do_tweak(
        auth.clone(),
        MarkerTweakRequest {
            marker_ids: vec![marker_id],
            tweaks: vec![MarkerTweakConfig {
                meta: TweakMeta {
                    item_list: Some(vec![Some(serde_json::json!(1003))]),
                    map: None,
                    replace: None,
                    test: None,
                    value: None,
                },
                prop: MarkerTweakConfigPropEnum::ItemList,
                marker_tweak_config_type: MarkerTweakConfigTypeEnum::RemoveLeft,
            }],
        },
    )
    .await
    .expect("remove tweak")
    .data
    .expect("tweak ok");
    assert_eq!(
        link_count(db, marker_id).await.expect("count"),
        0,
        "RemoveLeft removes the link"
    );

    // ── Assertion 5: punctuate audit enforces roles and commits atomically ───
    // Seed two Reviewing "Added" punctuates.
    seed_punctuate(db, 9001, MarkerPunctuateMethodType::Added, now)
        .await
        .expect("seed punctuate 9001");
    seed_punctuate(db, 9002, MarkerPunctuateMethodType::Added, now)
        .await
        .expect("seed punctuate 9002");
    let marker_before = marker_model::Entity::find_safety()
        .count(db)
        .await
        .expect("count markers");

    // Non-auditor role (MapUser) must be rejected for pass and reject
    let user_auth = stub_auth_with_role(SystemUserRole::MapUser);
    assert!(
        audit_fns::do_pass(user_auth.clone(), 9001).await.is_err(),
        "MapUser must not be allowed to pass an audit"
    );
    assert!(
        audit_fns::do_reject(user_auth, 9001, "nope".into())
            .await
            .is_err(),
        "MapUser must not be allowed to reject an audit"
    );

    // Admin rejects p2 → status becomes Rejected with the remark
    audit_fns::do_reject(auth.clone(), 9002, "bad data".into())
        .await
        .expect("admin rejects")
        .data
        .expect("reject ok");
    let rejected = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(9002))
        .one(db)
        .await
        .expect("fetch rejected")
        .expect("rejected punctuate still exists");
    assert_eq!(rejected.status, MarkerPunctuateStatus::Rejected);
    assert_eq!(rejected.audit_remark.as_deref(), Some("bad data"));

    // Admin passes 9001 → marker inserted AND punctuate record hard-deleted
    audit_fns::do_pass(auth.clone(), 9001)
        .await
        .expect("admin passes")
        .data
        .expect("pass ok");
    let marker_after = marker_model::Entity::find_safety()
        .count(db)
        .await
        .expect("count markers after pass");
    assert_eq!(
        marker_after,
        marker_before + 1,
        "pass must insert exactly one marker"
    );
    let p1_gone = mp_model::Entity::find_safety()
        .filter(mp_model::Column::PunctuateId.eq(9001))
        .one(db)
        .await
        .expect("check punctuate after pass");
    assert!(
        p1_gone.is_none(),
        "pass must hard-delete the punctuate record"
    );

    // ── Assertion 6: oauth access_policy checks + device registration ────────
    let ip_a = "1.2.3.4:5678".parse::<std::net::SocketAddr>().unwrap();
    let ip_b = "9.9.9.9:5678".parse::<std::net::SocketAddr>().unwrap();
    let ua = "test-agent/1.0";

    // 1) same_last_ip: first login from ip_a succeeds and registers the device;
    //    a second login from ip_b is rejected.
    seed_user(
        db,
        "policy_same_ip",
        vec![AccessPolicyItemEnum::IpSameLastIp],
        None,
        now,
    )
    .await
    .expect("seed policy user");
    oauth_fns::oauth_password_login("policy_same_ip".into(), "pw123".into(), ip_a, ua.into())
        .await
        .expect("first login from ip_a succeeds");
    let device = device_model::Entity::find_safety()
        .filter(device_model::Column::DeviceId.eq(ua))
        .one(db)
        .await
        .expect("fetch registered device")
        .expect("device registered after login");
    assert_eq!(device.ipv4.as_deref(), Some("1.2.3.4"));
    assert!(
        oauth_fns::oauth_password_login("policy_same_ip".into(), "pw123".into(), ip_b, ua.into())
            .await
            .is_err(),
        "same_last_ip policy must reject a different IP"
    );

    // 2) dev:block_disallow_device: a disabled device entry blocks login.
    let seed_user2 = seed_user(
        db,
        "policy_block_dev",
        vec![AccessPolicyItemEnum::DevBlockDisallowDevice],
        None,
        now,
    )
    .await
    .expect("seed blocked-device user");
    let blocked_am = device_model::ActiveModel {
        id: NotSet,
        version: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        user_id: Set(Some(seed_user2)),
        device_id: Set("evil-agent".into()),
        ipv4: Set(None),
        status: Set(1),
        last_login_time: Set(None),
    };
    device_model::Entity::insert(blocked_am)
        .exec(db)
        .await
        .expect("seed disabled device");
    assert!(
        oauth_fns::oauth_password_login(
            "policy_block_dev".into(),
            "pw123".into(),
            ip_a,
            "evil-agent".into(),
        )
        .await
        .is_err(),
        "dev:block_disallow_device must reject a blocked device"
    );

    // 3) scope mapping: "all"/"" → All, unknown → error.
    assert!(matches!(
        oauth_fns::map_scope("all").expect("all"),
        _utils::types::auth::OauthScopeType::All
    ));
    assert!(matches!(
        oauth_fns::map_scope("").expect("empty"),
        _utils::types::auth::OauthScopeType::All
    ));
    assert!(
        oauth_fns::map_scope("write").is_err(),
        "unknown scope must be rejected"
    );

    // ── Assertion 7: JWKS publishes the active signing key. With the
    //    ephemeral RSA key configured above, this is the RSA public key
    //    (kty=RSA with base64url n/e); the module also supports the HS256
    //    oct form when no RSA key is configured. ──────────────────────────────
    let jwks = oauth_fns::do_jwks().await.expect("do_jwks");
    let key = &jwks["keys"][0];
    assert_eq!(key["kty"], "RSA", "RS256 mode publishes an RSA key");
    assert_eq!(key["alg"], "RS256");
    assert_eq!(key["use"], "sig");
    let n = key["n"].as_str().expect("n is a string");
    let e = key["e"].as_str().expect("e is a string");
    use base64::Engine;
    assert!(
        !base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(n)
            .expect("n is valid base64url")
            .is_empty(),
        "n must decode to the modulus"
    );
    assert!(
        !base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(e)
            .expect("e is valid base64url")
            .is_empty(),
        "e must decode to the exponent"
    );
    // The RSA round-trip is proven by every login above (password + QQ):
    // tokens were RS256-signed and verified through the same key material.

    // ── Assertion 8: QQ login resolves the bound openid ──────────────────────
    seed_user(db, "qq_user", vec![], Some("OPENID_123".into()), now)
        .await
        .expect("seed qq user");

    // Registered openid → token issued
    let resp = oauth_fns::oauth_qq_login("OPENID_123".into(), ip_a, ua.into())
        .await
        .expect("qq login succeeds for a bound openid");
    assert!(!resp.access_token.is_empty());

    // Unregistered openid → error
    assert!(
        oauth_fns::oauth_qq_login("OPENID_UNKNOWN".into(), ip_a, ua.into())
            .await
            .is_err(),
        "unregistered openid must fail"
    );

    // ── Assertion 9: score generation weights by field count ────────────────
    // Seed three history rows: two Position (打点) rows with 3-field and
    // 1-field content, and one Area row that must be filtered out.
    let now_naive = now;
    for (i, (content, history_type, edit_type)) in [
        (
            r#"{"title":"a","position":"1,2","content":"c"}"#,
            HistoryOperationType::Position,
            HistoryEditType::Added,
        ),
        (
            r#"{"title":"b"}"#,
            HistoryOperationType::Position,
            HistoryEditType::Modified,
        ),
        (
            r#"{"name":"area-x"}"#,
            HistoryOperationType::Area,
            HistoryEditType::Added,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let am = history_model::ActiveModel {
            id: NotSet,
            version: Set(0),
            create_time: Set(now_naive + chrono::Duration::seconds(i as i64)),
            update_time: Set(None),
            creator_id: Set(Some(77)),
            updater_id: Set(None),
            del_flag: Set(false),
            content: Set(content.to_string()),
            md5: Set(None),
            t_id: Set(i as i64),
            history_type: Set(Some(history_type)),
            ipv4: Set(None),
            edit_type: Set(edit_type),
        };
        history_model::Entity::insert(am)
            .exec(db)
            .await
            .expect("seed history");
    }

    let start_ms = (now_naive - chrono::Duration::days(1))
        .and_utc()
        .timestamp_millis() as f64;
    let end_ms = (now_naive + chrono::Duration::days(1))
        .and_utc()
        .timestamp_millis() as f64;

    score_fns::do_generate_score(
        auth.clone(),
        ScoreGenerateRequest {
            end_time: end_ms,
            scope: "map".into(),
            span: "DAY".into(),
            start_time: start_ms,
        },
    )
    .await
    .expect("generate score")
    .data
    .expect("generate score ok");

    // Only the Position rows count; the 3-field row weights 3, the 1-field
    // row weights 1 → total 4 for creator 77.
    let stats = score_stat_model::Entity::find_safety()
        .filter(score_stat_model::Column::UserId.eq(Some(77)))
        .all(db)
        .await
        .expect("fetch score stats");
    assert_eq!(stats.len(), 1, "one score_stat row for the contributor");
    let content = stats[0].content.as_ref().expect("score content set");
    assert_eq!(content["count"], 2, "two Position edits counted");
    assert_eq!(
        content["fieldWeight"], 4.0,
        "field-weighted score = 3 (3 fields) + 1 (1 field)"
    );

    // do_get_score_data reads the real score (not a fixed 1.0).
    let data = score_fns::do_get_score_data(
        auth,
        ScoreDataRequest {
            end_time: end_ms,
            scope: "map".into(),
            span: "DAY".into(),
            start_time: start_ms,
        },
    )
    .await
    .expect("get score data")
    .data
    .expect("get score data ok");
    assert_eq!(data.samples.len(), 1, "one sample returned");
    assert_eq!(data.samples[0].score, 4.0, "score read back from content");
}
