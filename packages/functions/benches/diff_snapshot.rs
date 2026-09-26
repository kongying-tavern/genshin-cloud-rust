//! Benchmarks for the `/api/marker_doc/list_diff_snapshot` pipeline.
//!
//! The endpoint is called at high frequency by every client on map load, so
//! both recompute cost (DB round trips + CPU) and serving cost (cache hit
//! path) matter. The pipeline has two CPU stages — assembling snapshot rows
//! with their linkage groups, and hand-encoding the `MarkerDiffSnapshotVoList`
//! protobuf — plus one DB-backed stage that this bench seeds and measures.
//!
//! ```text
//! cargo bench -p _functions --bench diff_snapshot
//! # CI quick mode (also fine locally):
//! cargo bench -p _functions --bench diff_snapshot -- --warm-up-time 1 --measurement-time 2 --sample-size 10
//! ```
//!
//! The DB group self-skips exactly like the `*_db` integration tests: set
//! `GCS_TEST_DB=1` with a reachable Postgres (see tests/docker/
//! docker-compose.e2e.yml) to enable it.

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};

use _functions::functions::api::marker_doc::{
    DiffSnapshot, assemble_snapshots, diff_snapshot_bytes, encode_diff_snapshot_list,
};

/// Deterministic LCG so bench data is reproducible run to run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// Marker rows shaped like the JOIN query output: ascending ids in the
/// snowflake-ish range (6-byte varints), versions cycling 1..=9.
fn synthetic_markers(n: usize) -> Vec<(i64, i64)> {
    (0..n)
        .map(|i| (1 + (i % 9) as i64, 4_000_000_000_000 + i as i64))
        .collect()
}

/// Linkage triples: ~2/3 touch the visible marker set, 1/3 hang off it, with
/// 36-char group ids so field 15 has a realistic wire footprint.
fn synthetic_linkage_pairs(n: usize, markers: &[(i64, i64)]) -> Vec<(i64, i64, String)> {
    let mut rng = Lcg(0x5eed);
    (0..n)
        .map(|i| {
            let from = markers[(rng.next() as usize) % markers.len()].1;
            let to = markers[(rng.next() as usize) % markers.len()].1;
            let (from, to) = if i % 3 == 0 {
                (from + 9_000_000, to + 9_000_000)
            } else {
                (from, to)
            };
            (from, to, format!("550e8400-e29b-41d4-a716-44665544{i:04x}"))
        })
        .collect()
}

fn synthetic_snapshots(markers: &[(i64, i64)], pairs: &[(i64, i64, String)]) -> Vec<DiffSnapshot> {
    assemble_snapshots(markers.to_vec(), pairs)
}

fn cpu_benches(c: &mut Criterion) {
    for &n in &[10_000usize, 100_000] {
        let markers = synthetic_markers(n);
        let pairs = synthetic_linkage_pairs(n / 10, &markers);

        let mut group = c.benchmark_group("diff_snapshot");
        group.throughput(Throughput::Elements(n as u64));

        // Stage 1: rows + linkage triples → snapshot structs (hash joins).
        group.bench_function(format!("assemble/{n}"), |b| {
            b.iter_batched(
                || markers.clone(),
                |m| assemble_snapshots(m, &pairs),
                BatchSize::SmallInput,
            )
        });

        // Stage 2: snapshot structs → protobuf wire bytes.
        let snapshots = synthetic_snapshots(&markers, &pairs);
        group.bench_function(format!("encode/{n}"), |b| {
            b.iter(|| encode_diff_snapshot_list(&snapshots))
        });

        // Full CPU path of one recompute (what a cache miss pays beyond I/O).
        group.bench_function(format!("pipeline/{n}"), |b| {
            b.iter_batched(
                || markers.clone(),
                |m| encode_diff_snapshot_list(&assemble_snapshots(m, &pairs)),
                BatchSize::SmallInput,
            )
        });

        group.finish();
    }
}

// ---- DB-backed benches (GCS_TEST_DB gated) ----

use _database::{
    DB_CONN,
    models::{
        item::item as item_model,
        marker::{
            marker as marker_model, marker_item_link as mil_model, marker_linkage as linkage_model,
        },
    },
};
use _utils::types::{HiddenFlag, IconStyleType, MarkerLinkageLinkAction};
use sea_orm::{ActiveValue::Set, ConnectionTrait};

/// Build a CREATE TABLE statement with FOREIGN KEY constraints stripped, so
/// tables can be created without the sys_user dependency graph. Mirrors the
/// api_db_test helper of the same shape.
fn ddl_without_foreign_keys<E>(entity: E) -> String
where
    E: sea_orm::EntityTrait,
{
    use sea_orm::Schema;
    let schema = Schema::new(sea_orm::DbBackend::Postgres);
    let sql = schema
        .create_table_from_entity(entity)
        .to_string(sea_orm::sea_query::PostgresQueryBuilder);
    regex_lite::Regex::new(
        r#",(?:\s)*CONSTRAINT "fk-[^"]+" FOREIGN KEY \([^)]+\) REFERENCES (?:"[^"]+"\.)?"[^"]+" \([^)]+\)"#,
    )
    .expect("static regex")
    .replace_all(&sql, "")
    .into_owned()
}

/// Recreate the four tables the snapshot reads, FK-free, in the configured
/// schema, then seed `markers` markers (10% beta → invisible to visitors),
/// 1.3 links per marker, n/50 items and n/20 linkage groups.
async fn seed_snapshot_tables(
    db: &sea_orm::DatabaseConnection,
    markers: usize,
) -> anyhow::Result<()> {
    let schema = _database::default_schema();
    db.execute_unprepared(&format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#))
        .await?;
    for name in ["item", "marker", "marker_item_link", "marker_linkage"] {
        db.execute_unprepared(&format!(
            r#"DROP TABLE IF EXISTS "{schema}"."{name}" CASCADE"#
        ))
        .await?;
    }
    for ddl in [
        ddl_without_foreign_keys(item_model::Entity),
        ddl_without_foreign_keys(marker_model::Entity),
        ddl_without_foreign_keys(mil_model::Entity),
        ddl_without_foreign_keys(linkage_model::Entity),
    ] {
        db.execute_unprepared(&ddl).await?;
    }

    let now = chrono::Utc::now().naive_utc();
    let item_count = (markers / 50).max(2);

    let items: Vec<_> = (0..item_count)
        .map(|i| item_model::ActiveModel {
            version: Set(1),
            id: Set((i + 1) as i64),
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),
            name: Set(format!("bench-item-{i}")),
            area_id: Set(1),
            default_refresh_time: Set(0),
            default_content: Set(None),
            default_count: Set(1),
            icon_id: Set(0),
            icon_style_type: Set(IconStyleType::Default),
            hidden_flag: Set(if i % item_count == 0 {
                HiddenFlag::Beta
            } else {
                HiddenFlag::Visible
            }),
            sort_index: Set(0),
            special_flag: Set(None),
        })
        .collect();
    insert_chunked::<item_model::Entity>(db, items).await?;

    // 90% 可见 / 10% Beta（对访客不可见，让 JOIN 的 flag 过滤有活干）。
    let marker_rows: Vec<_> = (0..markers)
        .map(|i| marker_model::ActiveModel {
            version: Set(1 + (i % 9) as i64),
            id: Set(4_000_000_000_000 + i as i64),
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),
            marker_stamp: Set(None),
            marker_title: Set(None),
            position: Set("1.0,2.0".to_string()),
            content: Set(None),
            picture: Set(None),
            marker_creator_id: Set(1),
            picture_creator_id: Set(None),
            video_path: Set(None),
            refresh_time: Set(0),
            hidden_flag: Set(if i % 10 == 9 {
                HiddenFlag::Beta
            } else {
                HiddenFlag::Visible
            }),
            extra: Set(None),
        })
        .collect();
    insert_chunked::<marker_model::Entity>(db, marker_rows).await?;

    // 每个点位 1~2 条关联；物品随机取（含唯一的 Beta 物品，让物品侧
    // 的 flag 过滤有真实命中）。
    let mut rng = Lcg(0x5eed);
    let mut mil_rows = Vec::with_capacity(markers * 13 / 10);
    for i in 0..markers {
        let marker_id = 4_000_000_000_000 + i as i64;
        let links = 1 + usize::from(rng.next().is_multiple_of(2));
        for _ in 0..links {
            let item_idx = (rng.next() as usize) % item_count;
            mil_rows.push(mil_model::ActiveModel {
                version: Set(1),
                id: Set(9_000_000_000_000 + mil_rows.len() as i64),
                create_time: Set(now),
                update_time: Set(None),
                creator_id: Set(None),
                updater_id: Set(None),
                del_flag: Set(false),
                item_id: Set((item_idx + 1) as i64),
                marker_id: Set(marker_id),
                count: Set(1),
            });
        }
    }
    insert_chunked::<mil_model::Entity>(db, mil_rows).await?;

    let linkage_rows: Vec<_> = (0..markers / 20)
        .map(|i| {
            let from = 4_000_000_000_000 + (i * 2) as i64;
            let to = 4_000_000_000_000 + (i * 2 + 1) as i64;
            linkage_model::ActiveModel {
                version: Set(1),
                id: Set((i + 1) as i64),
                create_time: Set(now),
                update_time: Set(None),
                creator_id: Set(None),
                updater_id: Set(None),
                del_flag: Set(false),
                group_id: Set(format!("550e8400-e29b-41d4-a716-44665544{i:04x}")),
                from_id: Set(from),
                to_id: Set(to),
                link_action: Set(MarkerLinkageLinkAction::Trigger),
                link_reverse: Set(false),
                path: Set(None),
                extra: Set(None),
            }
        })
        .collect();
    insert_chunked::<linkage_model::Entity>(db, linkage_rows).await?;

    // 与实体声明对齐的二级索引（marker_item_link.item_id / .marker_id、
    // 两个 hidden_flag），镜像生产 schema 的查询形态。
    for idx in [
        r#"CREATE INDEX "bench_mil_item_idx" ON "{schema}"."marker_item_link" ("item_id")"#,
        r#"CREATE INDEX "bench_mil_marker_idx" ON "{schema}"."marker_item_link" ("marker_id")"#,
        r#"CREATE INDEX "bench_marker_hidden_idx" ON "{schema}"."marker" ("hidden_flag")"#,
        r#"CREATE INDEX "bench_item_hidden_idx" ON "{schema}"."item" ("hidden_flag")"#,
    ] {
        db.execute_unprepared(&idx.replace("{schema}", &schema))
            .await?;
    }
    Ok(())
}

/// `insert_many` in ≤2000-row statements so the generated SQL stays bounded.
async fn insert_chunked<E>(
    db: &sea_orm::DatabaseConnection,
    rows: Vec<E::ActiveModel>,
) -> anyhow::Result<()>
where
    E: sea_orm::EntityTrait,
    E::ActiveModel: sea_orm::ActiveModelTrait + Send,
{
    for chunk in rows.chunks(2000) {
        E::insert_many(chunk.to_vec()).exec(db).await?;
    }
    Ok(())
}

fn db_benches(c: &mut Criterion) {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!("skipped DB benches: set GCS_TEST_DB=1 with a reachable Postgres to run");
        return;
    }
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime for DB benches");
    let db = rt.block_on(async {
        if DB_CONN.get().is_none() {
            let _ = _database::init_db_conn().await;
        }
        DB_CONN.get().map(|m| &m.pg_conn)
    });
    let Some(db) = db else {
        eprintln!("skipped DB benches: DB connection unavailable");
        return;
    };

    for &n in &[10_000usize, 50_000] {
        rt.block_on(seed_snapshot_tables(db, n))
            .expect("seed snapshot bench tables");
        let mut group = c.benchmark_group("diff_snapshot_bytes");
        group.throughput(Throughput::Elements(n as u64));
        group.sample_size(10);
        // 访客可见集 [Visible, Suprise]；含 Beta 行让过滤路径被真实执行。
        group.bench_function(format!("db/{n}"), |b| {
            b.iter(|| rt.block_on(diff_snapshot_bytes(db, &[0, 3])))
        });
        group.finish();
    }
}

criterion_group!(benches, cpu_benches, db_benches);
criterion_main!(benches);
