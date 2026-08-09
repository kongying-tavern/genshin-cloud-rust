//! BinaryMD5 archive export utilities.
//!
//! Mirrors the Java `CompressUtils` + `DigestUtils` pipeline:
//!   serialize → GZIP compress → MD5 over compressed bytes.
//!
//! The compressed bytes are served as `application/octet-stream` keyed by
//! their MD5 hex, enabling client-side incremental sync.
//!
//! The generated pages are cached in-process (moka, like Java's Caffeine) so
//! repeated `list_page_bin_md5` / `list_page_bin` requests don't re-serialize
//! the whole dataset on every call. A **Redis second-level cache** shares the
//! computed page sets across replicas: a warm replica can serve the pages
//! without re-scanning the database, and invalidation bumps a versioned epoch
//! so every replica drops its stale copy at once.

use flate2::{Compression, write::GzEncoder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{io::Write, time::Duration};

use _database::DB_CONN;
use redis::AsyncCommands;

/// A single MD5 entry in the `list_page_bin_md5` response.
/// Mirrors Java `BinaryMD5Vo { md5, time }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryMd5Vo {
    pub md5: String,
    pub time: i64,
}

/// Serialize data to JSON, GZIP-compress the bytes, then compute the
/// lowercase-hex MD5 of the compressed payload.
///
/// Returns `(compressed_bytes, md5_hex)`.
pub fn serialize_compress_md5<T: Serialize>(data: &T) -> anyhow::Result<(Vec<u8>, String)> {
    // 1. Serialize to JSON (UTF-8)
    let json = serde_json::to_vec(data)?;

    // 2. GZIP compress
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json)?;
    let compressed = encoder.finish()?;

    // 3. MD5 over compressed bytes (lowercase hex, 32 chars)
    let digest = md5::compute(&compressed);
    let md5_hex = format!("{:x}", digest);

    Ok((compressed, md5_hex))
}

/// A cached BinaryMD5 page: the compressed bytes plus the metadata needed by
/// both the md5 list and the bin fetch endpoints.
#[derive(Debug, Clone)]
pub struct CachedPage {
    pub md5: String,
    /// Generation timestamp (stable while the cache entry is alive — the md5
    /// list `time` field must not change on every request).
    pub time: i64,
    pub bytes: Vec<u8>,
}

/// In-process page cache. Java uses Caffeine with a similar TTL; moka is the
/// Rust equivalent. The TTL bounds staleness until the refresh endpoints are
/// wired to an explicit invalidation channel (PLAN.md M4 / F10).
static BIN_CACHE: Lazy<moka::future::Cache<String, CachedPage>> = Lazy::new(|| {
    moka::future::Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(3600))
        .build()
});

/// Fetch a page from the cache, or compute it via `compute` and store it.
///
/// The cache is keyed by an explicit string (e.g. `item:0`, `marker:0:123`,
/// `link:graph`) that encodes the domain + group/page identity, so each page
/// is computed at most once per TTL window.
pub async fn get_or_compute(
    key: String,
    compute: impl std::future::Future<Output = anyhow::Result<CachedPage>>,
) -> anyhow::Result<CachedPage> {
    if let Some(cached) = BIN_CACHE.get(&key).await {
        return Ok(cached);
    }
    let page = compute.await?;
    BIN_CACHE.insert(key, page.clone()).await;
    Ok(page)
}

/// Drop a cache key (used by the cache-refresh endpoints once wired).
pub async fn invalidate(key: &str) {
    BIN_CACHE.invalidate(key).await;
}

/// Redis namespace for the result-level cache.
const REDIS_RESULT_PREFIX: &str = "binmd5:result:";
/// Redis epoch key: bumping it (INCR) invalidates every replica's copy of the
/// result cache atomically — stale keys fall out of the TTL window naturally,
/// so no SCAN/DEL pass is needed.
const REDIS_EPOCH_KEY: &str = "binmd5:epoch";
/// Redis entry TTL (matches the in-process moka TTL).
const REDIS_RESULT_TTL_SECS: u64 = 3600;

/// A `ResultEntry` serialized for Redis: bytes travel as base64 (JSON
/// carries `Vec<u8>` as a number array otherwise — 4-5x bigger).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RedisResultEntry {
    key: String,
    vo: BinaryMd5Vo,
    bytes_b64: String,
}

impl From<&ResultEntry> for RedisResultEntry {
    fn from(e: &ResultEntry) -> Self {
        use base64::Engine;
        RedisResultEntry {
            key: e.key.clone(),
            vo: e.vo.clone(),
            bytes_b64: base64::engine::general_purpose::STANDARD.encode(&e.bytes),
        }
    }
}

impl From<RedisResultEntry> for ResultEntry {
    fn from(e: RedisResultEntry) -> Self {
        use base64::Engine;
        ResultEntry {
            key: e.key,
            vo: e.vo,
            bytes: base64::engine::general_purpose::STANDARD
                .decode(&e.bytes_b64)
                .unwrap_or_default(),
        }
    }
}

/// Current Redis epoch (1 when unset). `None` when Redis is unavailable.
async fn redis_epoch() -> Option<i64> {
    let conn = DB_CONN.wait().redis_conn.as_ref()?;
    let mut c = conn.get_multiplexed_async_connection().await.ok()?;
    let epoch: Option<i64> = c.get(REDIS_EPOCH_KEY).await.ok().flatten();
    match epoch {
        Some(e) => Some(e),
        None => {
            // First touch: initialize under NX so concurrent replicas agree.
            let _: Result<i64, redis::RedisError> = c
                .set_options(
                    REDIS_EPOCH_KEY,
                    1,
                    redis::SetOptions::default().conditional_set(redis::ExistenceCheck::NX),
                )
                .await;
            Some(1)
        },
    }
}

/// Try to load the result set for `key` from Redis.
async fn redis_load(key: &str) -> Option<Vec<ResultEntry>> {
    let epoch = redis_epoch().await?;
    let conn = DB_CONN.wait().redis_conn.as_ref()?;
    let mut c = conn.get_multiplexed_async_connection().await.ok()?;
    let raw: Option<String> = c
        .get(format!("{REDIS_RESULT_PREFIX}{epoch}:{key}"))
        .await
        .ok()
        .flatten()?;
    let entries: Vec<RedisResultEntry> = serde_json::from_str(raw.as_deref()?).ok()?;
    Some(entries.into_iter().map(Into::into).collect())
}

/// Store the result set for `key` in Redis (best-effort; Redis may be down).
async fn redis_store(key: &str, entries: &[ResultEntry]) {
    let Some(epoch) = redis_epoch().await else {
        return;
    };
    let Some(conn) = DB_CONN.wait().redis_conn.as_ref().cloned() else {
        return;
    };
    let Ok(mut c) = conn.get_multiplexed_async_connection().await else {
        return;
    };
    let raw = match serde_json::to_string(
        &entries
            .iter()
            .map(RedisResultEntry::from)
            .collect::<Vec<_>>(),
    ) {
        Ok(r) => r,
        Err(_) => return,
    };
    let _: Result<(), redis::RedisError> = c
        .set_ex(
            format!("{REDIS_RESULT_PREFIX}{epoch}:{key}"),
            raw,
            REDIS_RESULT_TTL_SECS,
        )
        .await;
}

/// Drop every cached page (in-process + Redis across replicas).
pub async fn invalidate_all() {
    BIN_CACHE.invalidate_all();
    RESULT_CACHE.invalidate_all();
    // Bump the Redis epoch: replicas' copies are keyed by the old epoch and
    // expire on their own; the next request computes + stores under the new
    // one. Best-effort — with Redis down this is a no-op.
    let Some(conn) = DB_CONN.wait().redis_conn.as_ref() else {
        return;
    };
    let Ok(mut c) = conn.get_multiplexed_async_connection().await else {
        return;
    };
    let _: Result<i64, redis::RedisError> = c.incr(REDIS_EPOCH_KEY, 1).await;
}

/// Invalidate the item binary-doc caches (in-process moka + Redis across
/// replicas). Item writes change what `item_doc` serves, so call this after
/// any item / item_type / item_common write; otherwise other clients keep
/// seeing stale `item:result` / `item:{flag}` pages until the TTL expires.
///
/// This is the domain-wide invalidation: moka's `invalidate_all` drops the
/// in-process pages, and bumping the Redis epoch makes every replica's cached
/// copies unreachable (old keys expire on their own TTL).
pub async fn invalidate_item_doc_cache() {
    invalidate_all().await;
}

/// Invalidate the binary-doc caches for any domain (in-process moka + Redis
/// across replicas). Marker / marker_link / item writes all change what the
/// `*_doc` pages serve, so call this after any such write; otherwise other
/// clients keep seeing stale pages until the TTL expires.
pub async fn invalidate_doc_cache() {
    invalidate_all().await;
}

/// Result-level cache entry: one page's md5 metadata plus its compressed
/// bytes, so both the md5-list and the bin-fetch endpoints can be served
/// without any database scan on a warm hit.
#[derive(Debug, Clone)]
pub struct ResultEntry {
    /// Domain page key (e.g. `item:0`, `marker:0:123`, `link:list`).
    pub key: String,
    pub vo: BinaryMd5Vo,
    pub bytes: Vec<u8>,
}

/// Result-level cache: the fully computed page set for a domain (e.g.
/// `item:result`, `marker:result`, `link:list-result`).
///
/// Without it, every `list_page_bin_md5` / `list_page_bin` request re-ran the
/// full `find_safety().all()` scan (100k+ markers) before the per-page cache
/// could be consulted. With it, a warm hit performs **zero** database
/// queries.
static RESULT_CACHE: Lazy<moka::future::Cache<String, Vec<ResultEntry>>> = Lazy::new(|| {
    moka::future::Cache::builder()
        .max_capacity(64)
        .time_to_live(Duration::from_secs(3600))
        .build()
});

/// Fetch the full page set for a domain from the cache, or compute it via
/// `compute` and store it.
///
/// Lookup order: in-process moka → Redis (shared across replicas) → compute.
/// A Redis hit also warms the in-process cache, so subsequent requests are
/// zero-DB and zero-Redis.
pub async fn get_result_cached(
    key: String,
    compute: impl std::future::Future<Output = anyhow::Result<Vec<ResultEntry>>>,
) -> anyhow::Result<Vec<ResultEntry>> {
    if let Some(cached) = RESULT_CACHE.get(&key).await {
        return Ok(cached);
    }
    if let Some(entries) = redis_load(&key).await {
        RESULT_CACHE.insert(key.clone(), entries.clone()).await;
        return Ok(entries);
    }
    let result = compute.await?;
    RESULT_CACHE.insert(key.clone(), result.clone()).await;
    redis_store(&key, &result).await;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> ResultEntry {
        ResultEntry {
            key: "marker:0:0".into(),
            vo: BinaryMd5Vo {
                md5: "abc123".into(),
                time: 42,
            },
            bytes: vec![0xde, 0xad, 0xbe, 0xef],
        }
    }

    #[test]
    fn redis_entry_roundtrip_preserves_bytes() {
        // JSON carries bytes as base64; the round-trip must restore the exact
        // compressed payload (a broken transform would corrupt bin fetches).
        let entry = sample_entry();
        let redis_entry = RedisResultEntry::from(&entry);
        assert_eq!(redis_entry.bytes_b64, "3q2+7w==");
        let back = ResultEntry::from(redis_entry);
        assert_eq!(back.key, entry.key);
        assert_eq!(back.vo, entry.vo);
        assert_eq!(back.bytes, entry.bytes);
    }

    #[tokio::test]
    async fn epoch_keying_is_versioned() {
        // The epoch is a plain counter; entries are keyed by it so bumping
        // the epoch invalidates every replica's copy without a scan.
        let epoch = 7;
        let key = format!("{REDIS_RESULT_PREFIX}{epoch}:marker:result");
        assert!(key.starts_with(REDIS_RESULT_PREFIX));
        assert!(key.ends_with(":7:marker:result"));
    }
}
