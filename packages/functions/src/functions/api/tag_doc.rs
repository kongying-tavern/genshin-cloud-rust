//! BinaryMD5 archive export for icon tags.
//!
//! Mirrors Java `TagDocController` / `IconTagDao`: the whole tag set is one
//! GZIP-compressed JSON blob (each tag carrying its `typeIdList` from
//! `tag_type_link` and the icon `url`), keyed by the MD5 of the compressed
//! bytes. Cached at the result level — warm requests do no DB scan.

use anyhow::{Result, anyhow};
use sea_orm::{ColumnTrait, QueryFilter};
use serde::Serialize;

use _database::{
    DB_CONN,
    models::{icon::icon as icon_model, tag::tag as tag_model, tag::tag_type_link as ttl_model},
};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{
    BinaryMd5Vo, CachedPage, ResultEntry, get_or_compute, get_result_cached, serialize_compress_md5,
};

/// camelCase tag view for the BinaryMD5 blob (Java `TagVo` naming).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDocVo {
    pub version: i64,
    pub creator_id: Option<i64>,
    pub create_time: chrono::NaiveDateTime,
    pub updater_id: Option<i64>,
    pub update_time: Option<chrono::NaiveDateTime>,
    pub tag: String,
    pub type_id_list: Vec<i64>,
    pub icon_id: i64,
    pub url: Option<String>,
}

/// `GET /tag_doc/all_bin_md5` — MD5 of the single all-tags blob.
pub async fn do_all_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let entry = tag_result().await?;
    Ok(CommonResponse::new(Ok(entry.vo)))
}

/// `GET /tag_doc/all_bin` — the all-tags blob (compressed bytes).
pub async fn do_all_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let entry = tag_result().await?;
    Ok(entry.bytes)
}

/// Compute (and cache) the single tag blob.
async fn tag_result() -> Result<ResultEntry> {
    let db = &DB_CONN.wait().pg_conn;

    let entries = get_result_cached("tag:result".into(), async {
        let tags = tag_model::Entity::find_safety().all(db).await?;

        // typeIdList per tag (tag_type_link is keyed by tag_name).
        let mut type_map: std::collections::BTreeMap<String, Vec<i64>> =
            std::collections::BTreeMap::new();
        for link in ttl_model::Entity::find_safety().all(db).await? {
            type_map
                .entry(link.tag_name)
                .or_default()
                .push(link.type_id);
        }

        // icon url per icon id.
        let icon_ids: Vec<i64> = tags.iter().map(|t| t.icon_id).collect();
        let mut url_map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
        if !icon_ids.is_empty() {
            for icon in icon_model::Entity::find_safety()
                .filter(icon_model::Column::Id.is_in(icon_ids))
                .all(db)
                .await?
            {
                let base = std::env::var("ICON_PROXY_BASE").unwrap_or_default();
                let url = if icon.url.contains("ddns.minemc.top") && !base.is_empty() {
                    format!("{base}/cdn/img-proxy?u={}", urlencoding::encode(&icon.url))
                } else {
                    icon.url
                };
                url_map.insert(icon.id, url);
            }
        }

        let vos: Vec<TagDocVo> = tags
            .into_iter()
            .map(|t| {
                let type_id_list = type_map.remove(&t.tag).unwrap_or_default();
                TagDocVo {
                    version: t.version,
                    creator_id: t.creator_id,
                    create_time: t.create_time,
                    updater_id: t.updater_id,
                    update_time: t.update_time,
                    tag: t.tag,
                    type_id_list,
                    icon_id: t.icon_id,
                    url: url_map.get(&t.icon_id).cloned(),
                }
            })
            .collect();

        let (compressed, md5_hex) = serialize_compress_md5(&vos)?;
        let page = get_or_compute("tag:all".to_string(), async {
            Ok(CachedPage {
                md5: md5_hex,
                time: chrono::Utc::now().timestamp_millis(),
                bytes: compressed,
            })
        })
        .await?;
        Ok(vec![ResultEntry {
            key: "tag:all".to_string(),
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
        .ok_or_else(|| anyhow!("empty tag result"))
}
