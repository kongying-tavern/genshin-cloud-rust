//! BinaryMD5 archive export utilities.
//!
//! Mirrors the Java `CompressUtils` + `DigestUtils` pipeline:
//!   serialize → GZIP compress → MD5 over compressed bytes.
//!
//! The compressed bytes are served as `application/octet-stream` keyed by
//! their MD5 hex, enabling client-side incremental sync.

use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::io::Write;

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
