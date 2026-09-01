//! BinaryMD5 archive export for marker linkages.
//!
//! Mirrors Java `MarkerLinkageDaoImpl.listAllMarkerLinkage` /
//! `graphAllMarkerLinkage`. Single-blob shape (no per-flag paging): the entire
//! dataset is one GZIP-compressed JSON blob. Two views, both byte-compatible
//! with the Java wire contract:
//! - `list`: `{groupId: [MarkerLinkageVo]}`（已 reverse + 路径坐标回填）
//! - `graph`: `{groupId: GraphVo{relations, relRefs, pathRefs}}`
//!
//! Both are cached at the result level, so warm requests perform no database
//! scan.

use anyhow::{Result, anyhow};

use _database::{DB_CONN, models::marker::marker_linkage as ml_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{
    BinaryMd5Vo, CachedPage, ResultEntry, get_or_compute, get_result_cached, serialize_compress_md5,
};
use super::marker_link::build_linkage_graph;
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

/// `GET /marker_link_doc/all_graph_bin_md5` — MD5 of the graph blob.
pub async fn do_all_graph_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let entry = linkage_result("link:graph-result", true).await?;
    Ok(CommonResponse::new(Ok(entry.vo)))
}

/// `GET /marker_link_doc/all_graph_bin` — the graph blob (compressed bytes).
pub async fn do_all_graph_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let entry = linkage_result("link:graph-result", true).await?;
    Ok(entry.bytes)
}

/// Compute (and cache) one linkage blob view.
async fn linkage_result(key: &'static str, graph: bool) -> Result<ResultEntry> {
    let db = &DB_CONN.wait().pg_conn;

    let entries = get_result_cached(key.to_string(), async {
        let linkages = ml_model::Entity::find_safety().all(db).await?;
        // 读侧变换（Java getAllMarkerLinkage）：reverse + 路径坐标回填。
        // blob 内字段为 camelCase `MarkerLinkageVo` 命名（Java wire contract），
        // 前端解压后直接按 Record<string, MarkerLinkageVo[]> / GraphVo 消费。
        let mut vos: Vec<MarkerLinkVO> = linkages
            .into_iter()
            .map(super::marker_link::model_to_vo)
            .collect();
        super::marker_link::reverse_linkage_vos(&mut vos);
        {
            let path_ids = super::marker_link::path_marker_ids(&vos);
            let coords = super::marker_link::path_marker_coords(db, &path_ids).await?;
            super::marker_link::patch_path_coords(&mut vos, &coords);
        }
        let data: serde_json::Value = if graph {
            serde_json::to_value(build_linkage_graph(&vos))?
        } else {
            // BTreeMap → 键序确定，保证数据未变时序列化结果（MD5）稳定
            let mut map: std::collections::BTreeMap<String, Vec<MarkerLinkVO>> =
                std::collections::BTreeMap::new();
            for vo in vos {
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
