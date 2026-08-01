//! BinaryMD5 archive export for marker linkages.
//!
//! Mirrors Java `MarkerLinkageDocController`. Single-blob shape (no per-flag
//! paging): the entire dataset is one GZIP-compressed JSON blob.
//! Two views: `list` (flat array) and `graph` (adjacency map). Both are cached
//! in-process via `binary_doc::get_or_compute`.

use anyhow::Result;

use _database::{DB_CONN, models::marker::marker_linkage as ml_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{BinaryMd5Vo, CachedPage, get_or_compute, serialize_compress_md5};

/// `GET /marker_link_doc/all_list_bin_md5` — MD5 of the flat linkage list blob.
pub async fn do_all_list_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let db = &DB_CONN.wait().pg_conn;

    let page = get_or_compute("link:list".into(), async {
        let linkages = ml_model::Entity::find_safety().all(db).await?;
        let (compressed, md5_hex) = serialize_compress_md5(&linkages)?;
        Ok(CachedPage {
            md5: md5_hex,
            time: chrono::Utc::now().timestamp_millis(),
            bytes: compressed,
        })
    })
    .await?;

    Ok(CommonResponse::new(Ok(BinaryMd5Vo {
        md5: page.md5,
        time: page.time,
    })))
}

/// `GET /marker_link_doc/all_list_bin` — the flat linkage list blob (compressed bytes).
pub async fn do_all_list_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let db = &DB_CONN.wait().pg_conn;

    let page = get_or_compute("link:list".into(), async {
        let linkages = ml_model::Entity::find_safety().all(db).await?;
        let (compressed, md5_hex) = serialize_compress_md5(&linkages)?;
        Ok(CachedPage {
            md5: md5_hex,
            time: chrono::Utc::now().timestamp_millis(),
            bytes: compressed,
        })
    })
    .await?;
    Ok(page.bytes)
}

/// `GET /marker_link_doc/all_graph_bin_md5` — MD5 of the graph adjacency blob.
pub async fn do_all_graph_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let db = &DB_CONN.wait().pg_conn;

    let page = get_or_compute("link:graph".into(), async {
        let linkages = ml_model::Entity::find_safety().all(db).await?;
        let graph = build_graph(&linkages);
        let (compressed, md5_hex) = serialize_compress_md5(&graph)?;
        Ok(CachedPage {
            md5: md5_hex,
            time: chrono::Utc::now().timestamp_millis(),
            bytes: compressed,
        })
    })
    .await?;

    Ok(CommonResponse::new(Ok(BinaryMd5Vo {
        md5: page.md5,
        time: page.time,
    })))
}

/// `GET /marker_link_doc/all_graph_bin` — the graph adjacency blob (compressed bytes).
pub async fn do_all_graph_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let db = &DB_CONN.wait().pg_conn;

    let page = get_or_compute("link:graph".into(), async {
        let linkages = ml_model::Entity::find_safety().all(db).await?;
        let graph = build_graph(&linkages);
        let (compressed, md5_hex) = serialize_compress_md5(&graph)?;
        Ok(CachedPage {
            md5: md5_hex,
            time: chrono::Utc::now().timestamp_millis(),
            bytes: compressed,
        })
    })
    .await?;
    Ok(page.bytes)
}

/// Build the adjacency map: marker_id → list of linked marker_ids.
fn build_graph(linkages: &[ml_model::Model]) -> std::collections::HashMap<i64, Vec<i64>> {
    let mut graph: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for l in linkages {
        graph.entry(l.from_id).or_default().push(l.to_id);
    }
    graph
}
