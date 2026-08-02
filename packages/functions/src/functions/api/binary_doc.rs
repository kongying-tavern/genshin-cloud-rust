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
//! the whole dataset on every call.

use flate2::{Compression, write::GzEncoder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{io::Write, time::Duration};

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
        .time_to_live(Duration::from_secs(300))
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

/// Drop every cached page. The cache-refresh endpoints call this — the keys
/// are domain-scoped (`item:*`, `marker:*`, `link:*`) but not enumerable
/// cheaply, so a full flush is the simple correct invalidation.
pub fn invalidate_all() {
    BIN_CACHE.invalidate_all();
    RESULT_CACHE.invalidate_all();
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
        .time_to_live(Duration::from_secs(300))
        .build()
});

/// Fetch the full page set for a domain from the cache, or compute it via
/// `compute` and store it.
pub async fn get_result_cached(
    key: String,
    compute: impl std::future::Future<Output = anyhow::Result<Vec<ResultEntry>>>,
) -> anyhow::Result<Vec<ResultEntry>> {
    if let Some(cached) = RESULT_CACHE.get(&key).await {
        return Ok(cached);
    }
    let result = compute.await?;
    RESULT_CACHE.insert(key, result.clone()).await;
    Ok(result)
}
