//! DB-backed integration tests for write-path integrity: soft delete and
//! restore, the optimistic-lock strict update, and transaction rollback.
//!
//! Gated on `GCS_TEST_DB` exactly like the other `*_db` suites (CI's
//! `integration` job sets it and provisions Postgres). Without the variable
//! (or without a reachable database) the test skips cleanly so
//! `cargo test --workspace` stays green on a bare machine.
//!
//! What it pins down, against a real PostgreSQL:
//!   1. `delete_safety` really hides a row from `find_safety`, and
//!      `restore_safety` brings it back (the pair the marker-link
//!      resurrection path relies on).
//!   2. `update_safety_strict` rejects a stale `version` with a
//!      `DomainError` instead of silently updating zero rows.
//!   3. `do_link` — link / remove / re-link — is consistent through
//!      `find_safety` (delete_safety + restore_safety under a transaction).
//!   4. A mid-batch CHECK-constraint violation inside `do_link` rolls the
//!      whole batch back (the first inserted pair must not survive).
//!   5. A failure in the middle of the `icon_type` BFS cascade rolls the
//!      subtree delete back; retrying after the failure removes the whole
//!      subtree and its icon_type_link rows consistently.
//!
//! The injected failures use temporary CHECK constraints on the scratch tables
//! (created FK-free from the entity DDL), so no production code path needs a
//! test-only hook.

use std::sync::Arc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Schema,
    sea_query::TableCreateStatement,
};
use tokio::sync::OnceCell;

use _database::DatabaseConnectionMap;
use _database::models::{
    icon::{icon_type as icon_type_model, icon_type_link as itl_model},
    marker::{marker as marker_model, marker_linkage as linkage_model},
};
use _functions::functions::api::{icon_type as icon_type_fns, marker_link as marker_link_fns};
use _utils::{
    db_operations::SafeEntityTrait,
    errors::DomainError,
    jwt::AuthInfo,
    models::{SysUserVO, marker_link::MarkerLinkage},
    types::{AccessPolicyList, HiddenFlag, MarkerLinkageLinkAction, SystemUserRole},
};

/// Self-owned connection map: this binary builds one connection set and never
/// touches the global `DB_CONN` singleton (test isolation contract).
static MAP: OnceCell<Option<Arc<DatabaseConnectionMap>>> = OnceCell::const_new();

async fn map() -> Option<&'static DatabaseConnectionMap> {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!("skipped: set GCS_TEST_DB=1 with a reachable Postgres to run");
        return None;
    }
    let arc = MAP
        .get_or_init(|| async {
            match _database::connect_db_map().await {
                Ok(m) => Some(Arc::new(m)),
                Err(e) => {
                    eprintln!("skipped: connect_db_map failed: {e}");
                    None
                },
            }
        })
        .await
        .as_ref()?;
    Some(arc)
}

/// Minimal admin AuthInfo (all hidden flags visible) for calling business
/// functions directly.
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

/// Entity DDL with FOREIGN KEY constraints stripped: these tests only need a
/// handful of tables and do not want to build the full dependency graph.
fn ddl_without_foreign_keys<E>(entity: E) -> String
where
    E: EntityTrait,
{
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let stmt: TableCreateStatement = schema.create_table_from_entity(entity);
    let sql = stmt.to_string(sea_orm::sea_query::PostgresQueryBuilder);
    let stripped = regex_lite::Regex::new(
        r#",(?:\s)*CONSTRAINT "fk-[^"]+" FOREIGN KEY \([^)]+\) REFERENCES (?:"[^"]+"\.)?"[^"]+" \([^)]+\)"#,
    )
    .expect("static regex")
    .replace_all(&sql, "");
    stripped.into_owned()
}

async fn recreate_tables(
    db: &sea_orm::DatabaseConnection,
    table_names: &[&str],
    ddls: &[String],
) -> anyhow::Result<()> {
    let schema = _database::default_schema();
    db.execute_unprepared(&format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#))
        .await?;
    for name in table_names {
        db.execute_unprepared(&format!(
            r#"DROP TABLE IF EXISTS "{schema}"."{name}" CASCADE"#
        ))
        .await?;
    }
    for ddl in ddls {
        db.execute_unprepared(ddl).await?;
    }
    Ok(())
}

/// Create the marker + marker_linkage scratch pair used by the link tests.
async fn recreate_link_tables(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    recreate_tables(
        db,
        &["marker_linkage", "marker"],
        &[
            ddl_without_foreign_keys(marker_model::Entity),
            ddl_without_foreign_keys(linkage_model::Entity),
        ],
    )
    .await
}

/// Create the icon_type + icon_type_link scratch pair used by the cascade tests.
async fn recreate_icon_type_tables(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    recreate_tables(
        db,
        &["icon_type_link", "icon_type"],
        &[
            ddl_without_foreign_keys(icon_type_model::Entity),
            ddl_without_foreign_keys(itl_model::Entity),
        ],
    )
    .await
}

async fn seed_marker(db: &sea_orm::DatabaseConnection, id: i64) -> anyhow::Result<()> {
    let now = chrono::Utc::now().naive_utc();
    marker_model::Entity::insert(marker_model::ActiveModel {
        version: Set(1),
        id: Set(id),
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(1)),
        updater_id: Set(Some(1)),
        del_flag: Set(false),
        marker_stamp: Set(None),
        marker_title: Set(Some(format!("marker-{id}"))),
        position: Set("0.0,0.0".into()),
        content: Set(None),
        picture: Set(None),
        marker_creator_id: Set(1),
        picture_creator_id: Set(None),
        video_path: Set(None),
        refresh_time: Set(0),
        hidden_flag: Set(HiddenFlag::Visible),
        extra: Set(None),
    })
    .exec(db)
    .await?;
    Ok(())
}

async fn seed_linkage(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    from_id: i64,
    to_id: i64,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().naive_utc();
    linkage_model::Entity::insert(linkage_model::ActiveModel {
        version: Set(1),
        id: Set(id),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        group_id: Set("g-write-integrity".into()),
        from_id: Set(from_id),
        to_id: Set(to_id),
        link_action: Set(MarkerLinkageLinkAction::Trigger),
        link_reverse: Set(false),
        path: Set(Some(serde_json::json!([]))),
        extra: Set(Some(serde_json::json!({}))),
    })
    .exec(db)
    .await?;
    Ok(())
}

async fn seed_icon_type(
    db: &sea_orm::DatabaseConnection,
    name: &str,
    parent_id: i64,
) -> anyhow::Result<i64> {
    let now = chrono::Utc::now().naive_utc();
    let res = icon_type_model::Entity::insert(icon_type_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        name: Set(name.into()),
        parent_id: Set(parent_id),
        is_final: Set(true),
    })
    .exec(db)
    .await?;
    Ok(res.last_insert_id)
}

async fn seed_icon_type_link(
    db: &sea_orm::DatabaseConnection,
    type_id: i64,
    icon_id: i64,
) -> anyhow::Result<i64> {
    let now = chrono::Utc::now().naive_utc();
    let res = itl_model::Entity::insert(itl_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        type_id: Set(type_id),
        icon_id: Set(icon_id),
    })
    .exec(db)
    .await?;
    Ok(res.last_insert_id)
}

async fn raw_linkage(
    db: &sea_orm::DatabaseConnection,
    from_id: i64,
    to_id: i64,
) -> Option<linkage_model::Model> {
    linkage_model::Entity::find()
        .filter(linkage_model::Column::FromId.eq(from_id))
        .filter(linkage_model::Column::ToId.eq(to_id))
        .one(db)
        .await
        .expect("raw linkage query")
}

async fn linkage_visible(db: &sea_orm::DatabaseConnection, from_id: i64, to_id: i64) -> bool {
    linkage_model::Entity::find_safety()
        .filter(linkage_model::Column::FromId.eq(from_id))
        .filter(linkage_model::Column::ToId.eq(to_id))
        .count(db)
        .await
        .expect("visible linkage count")
        > 0
}

fn edge(from_id: i64, to_id: i64) -> MarkerLinkage {
    MarkerLinkage {
        from_id,
        id: None,
        link_action: Some(MarkerLinkageLinkAction::Trigger),
        path: Some(vec![]),
        to_id,
        link_reverse: None,
    }
}

#[tokio::test]
async fn write_integrity_round_trip() {
    let Some(map) = map().await else {
        return;
    };
    let db = &map.pg_conn;
    let schema = _database::default_schema();

    // ── (a) delete_safety / restore_safety round-trip ──────────────────────
    recreate_link_tables(db)
        .await
        .expect("recreate link tables");
    seed_linkage(db, 5001, 1001, 1002)
        .await
        .expect("seed linkage");
    assert!(
        linkage_visible(db, 1001, 1002).await,
        "seeded row must be visible through find_safety"
    );

    let model = linkage_model::Entity::find_by_id(5001i64)
        .one(db)
        .await
        .expect("read seeded row")
        .expect("seeded row must exist");
    linkage_model::Entity::delete_safety(model.into())
        .expect("build soft delete")
        .exec(db)
        .await
        .expect("soft delete");
    assert!(
        linkage_model::Entity::find_safety_by_id(5001)
            .one(db)
            .await
            .expect("read after delete")
            .is_none(),
        "soft-deleted row must be hidden from find_safety"
    );
    let deleted = linkage_model::Entity::find_by_id(5001i64)
        .one(db)
        .await
        .expect("read raw deleted row")
        .expect("row still exists");
    assert!(deleted.del_flag, "raw row must carry del_flag = true");

    linkage_model::Entity::restore_safety(deleted.into())
        .expect("build restore")
        .exec(db)
        .await
        .expect("restore");
    let restored = linkage_model::Entity::find_safety_by_id(5001)
        .one(db)
        .await
        .expect("read after restore")
        .expect("restored row must be visible again");
    assert!(
        !restored.del_flag,
        "restored row must have del_flag = false"
    );

    // ── (b) update_safety_strict rejects a stale version ──────────────────
    recreate_icon_type_tables(db)
        .await
        .expect("recreate icon type tables");
    let type_id = seed_icon_type(db, "strict-root", -1)
        .await
        .expect("seed icon type");
    let first = icon_type_model::Entity::find_safety_by_id(type_id)
        .one(db)
        .await
        .expect("read first copy")
        .expect("icon type exists");
    let stale = icon_type_model::Entity::find_safety_by_id(type_id)
        .one(db)
        .await
        .expect("read stale copy")
        .expect("icon type exists");

    let mut first_am: icon_type_model::ActiveModel = first.into();
    first_am.name = Set("strict-root-one".into());
    icon_type_model::Entity::update_safety_strict(first_am, db)
        .await
        .expect("first writer must win");

    let mut stale_am: icon_type_model::ActiveModel = stale.into();
    stale_am.name = Set("strict-root-stale".into());
    let err = icon_type_model::Entity::update_safety_strict(stale_am, db)
        .await
        .expect_err("stale writer must fail");
    assert!(
        err.chain()
            .any(|cause| cause.downcast_ref::<DomainError>().is_some()),
        "stale update must surface a DomainError, got: {err:?}"
    );
    let after = icon_type_model::Entity::find_safety_by_id(type_id)
        .one(db)
        .await
        .expect("read after conflict")
        .expect("icon type exists");
    assert_eq!(after.name, "strict-root-one", "losing write must not land");
    assert_eq!(after.version, 2, "winning write must bump the version once");

    // ── (c) do_link: link → remove → re-link (delete + restore) ────────────
    recreate_link_tables(db)
        .await
        .expect("recreate link tables");
    for id in [2001, 2002, 2003] {
        seed_marker(db, id).await.expect("seed marker");
    }
    marker_link_fns::do_link(map, stub_auth(), vec![edge(2001, 2002)])
        .await
        .expect("link 2001-2002");
    assert!(
        linkage_visible(db, 2001, 2002).await,
        "linked pair must be visible"
    );

    marker_link_fns::do_link(map, stub_auth(), vec![edge(2001, 2003)])
        .await
        .expect("re-link 2001 to 2003");
    assert!(
        !linkage_visible(db, 2001, 2002).await,
        "pair absent from the new batch must be soft-deleted"
    );
    assert!(
        linkage_visible(db, 2001, 2003).await,
        "new pair must be visible"
    );

    marker_link_fns::do_link(map, stub_auth(), vec![edge(2001, 2002)])
        .await
        .expect("re-add 2001-2002");
    assert!(
        linkage_visible(db, 2001, 2002).await,
        "restore_safety must resurrect the soft-deleted pair"
    );
    assert!(
        !linkage_visible(db, 2001, 2003).await,
        "pair absent from the new batch must be soft-deleted again"
    );

    // ── (d) do_link rolls the whole batch back on a mid-batch failure ──────
    db.execute_unprepared(&format!(
        r#"ALTER TABLE "{schema}"."marker_linkage"
           ADD CONSTRAINT "chk_wit_reject_2004" CHECK (to_id <> 2004)"#
    ))
    .await
    .expect("install failure-injection constraint");
    let err = marker_link_fns::do_link(map, stub_auth(), vec![edge(2001, 2005), edge(2001, 2004)])
        .await
        .expect_err("second row violates the injected constraint");
    assert!(
        err.chain()
            .any(|cause| cause.downcast_ref::<sea_orm::DbErr>().is_some()),
        "expected a DbErr from the injected constraint, got: {err:?}"
    );
    assert!(
        raw_linkage(db, 2001, 2005).await.is_none(),
        "first inserted pair must be rolled back with the failed batch"
    );
    assert!(
        linkage_visible(db, 2001, 2002).await,
        "pre-existing pair must survive the rolled-back batch"
    );
    db.execute_unprepared(&format!(
        r#"ALTER TABLE "{schema}"."marker_linkage" DROP CONSTRAINT "chk_wit_reject_2004""#
    ))
    .await
    .expect("drop failure-injection constraint");

    // ── (e) icon_type BFS cascade: rollback, then consistent delete ────────
    recreate_icon_type_tables(db)
        .await
        .expect("recreate icon type tables");
    let parent = seed_icon_type(db, "cascade-parent", -1)
        .await
        .expect("seed parent type");
    let child = seed_icon_type(db, "cascade-child", parent)
        .await
        .expect("seed child type");
    let parent_link = seed_icon_type_link(db, parent, 9001)
        .await
        .expect("seed parent link");
    let child_link = seed_icon_type_link(db, child, 9002)
        .await
        .expect("seed child link");

    db.execute_unprepared(&format!(
        r#"ALTER TABLE "{schema}"."icon_type_link"
           ADD CONSTRAINT "chk_wit_block_child_delete"
           CHECK (NOT (del_flag AND type_id = {child}))"#
    ))
    .await
    .expect("install cascade failure constraint");
    icon_type_fns::do_delete(map, stub_auth(), parent)
        .await
        .expect_err("cascade must fail on the child link");
    assert!(
        icon_type_model::Entity::find_safety_by_id(parent)
            .one(db)
            .await
            .expect("read parent after rollback")
            .is_some(),
        "rollback must leave the parent type visible"
    );
    let parent_link_after = itl_model::Entity::find_by_id(parent_link)
        .one(db)
        .await
        .expect("read parent link after rollback")
        .expect("parent link row still exists");
    assert!(
        !parent_link_after.del_flag,
        "rollback must undo the parent link soft delete"
    );
    db.execute_unprepared(&format!(
        r#"ALTER TABLE "{schema}"."icon_type_link" DROP CONSTRAINT "chk_wit_block_child_delete""#
    ))
    .await
    .expect("drop cascade failure constraint");

    icon_type_fns::do_delete(map, stub_auth(), parent)
        .await
        .expect("cascade delete after the constraint is gone");
    for (label, id) in [("parent", parent), ("child", child)] {
        assert!(
            icon_type_model::Entity::find_safety_by_id(id)
                .one(db)
                .await
                .expect("read type after cascade")
                .is_none(),
            "{label} type must be soft-deleted by the cascade"
        );
    }
    for id in [parent_link, child_link] {
        assert!(
            itl_model::Entity::find_safety_by_id(id)
                .one(db)
                .await
                .expect("read link after cascade")
                .is_none(),
            "icon_type_link {id} must be soft-deleted by the cascade"
        );
    }
}
