//! BinaryMD5 archive export for marker linkages.
//!
//! Mirrors Java `MarkerLinkageDocController`. Single-blob shape (no per-flag
//! paging): the entire dataset is one GZIP-compressed JSON blob.
//! Two views: `list` (grouped map `groupId -> [MarkerLinkageVo]`) and `graph`
//! (adjacency map). Both are cached at the result level, so warm requests
//! perform no database scan.

use anyhow::{Result, anyhow};

use _database::{DB_CONN, models::marker::marker_linkage as ml_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{
    BinaryMd5Vo, CachedPage, ResultEntry, get_or_compute, get_result_cached, serialize_compress_md5,
};
use super::marker_link::model_to_vo;
use _utils::models::marker_link::MarkerLinkVO;

/// `GET /marker_link_doc/all_list_bin_md5` — MD5 of the flat linkage list blob.
pub async fn do_all_list_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let entry = linkage_result("link:list-result", false).await?;
    Ok(CommonResponse::new(Ok(entry.vo)))
}

/// `GET /marker_link_doc/all_list_bin` — the flat linkage list blob (compressed bytes).
pub async fn do_all_list_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let entry = linkage_result("link:list-result", false).await?;
    Ok(entry.bytes)
}

/// `GET /marker_link_doc/all_graph_bin_md5` — MD5 of the graph adjacency blob.
pub async fn do_all_graph_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let entry = linkage_result("link:graph-result", true).await?;
    Ok(CommonResponse::new(Ok(entry.vo)))
}

/// `GET /marker_link_doc/all_graph_bin` — the graph adjacency blob (compressed bytes).
pub async fn do_all_graph_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let entry = linkage_result("link:graph-result", true).await?;
    Ok(entry.bytes)
}

/// Compute (and cache) one linkage blob view.
async fn linkage_result(key: &'static str, graph: bool) -> Result<ResultEntry> {
    let db = &DB_CONN.wait().pg_conn;

    let entries = get_result_cached(key.to_string(), async {
        let linkages = ml_model::Entity::find_safety().all(db).await?;
        let data: serde_json::Value = if graph {
            serde_json::to_value(build_graph(&linkages))?
        } else {
            // 按 group_id 分组（camelCase `MarkerLinkageVo` 命名，Java wire contract），
            // 前端解压后期望 `Record<string, MarkerLinkageVo[]>` 的 map
            // BTreeMap → 键序确定，保证数据未变时序列化结果（MD5）稳定
            let mut map: std::collections::BTreeMap<String, Vec<MarkerLinkVO>> =
                std::collections::BTreeMap::new();
            for l in linkages {
                let vo = model_to_vo(l);
                let group_id = vo.group_id.clone().unwrap_or_default();
                map.entry(group_id).or_default().push(vo);
            }
            serde_json::to_value(&map)?
        };
        let (compressed, md5_hex) = serialize_compress_md5(&data)?;
        let page = get_or_compute(key.to_string(), async {
            Ok(CachedPage {
                md5: md5_hex,
                time: chrono::Utc::now().timestamp_millis(),
                bytes: compressed,
            })
        })
        .await?;
        Ok(vec![ResultEntry {
            key: key.to_string(),
            vo: BinaryMd5Vo {
                md5: page.md5,
                time: page.time,
            },
            bytes: page.bytes,
        }])
    })
    .await?;

    entries
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("empty linkage result"))
}

/// Build the adjacency map: marker_id → list of linked marker_ids.
/// BTreeMap → 键序确定，保证序列化结果（MD5）稳定。
fn build_graph(linkages: &[ml_model::Model]) -> std::collections::BTreeMap<i64, Vec<i64>> {
    let mut graph: std::collections::BTreeMap<i64, Vec<i64>> = std::collections::BTreeMap::new();
    for l in linkages {
        graph.entry(l.from_id).or_default().push(l.to_id);
    }
    graph
}
