//! BinaryMD5 archive export for icons.
//!
//! Mirrors Java `IconDocController` / `IconDaoImpl`: the whole icon set is
//! one GZIP-compressed JSON blob (each icon carrying its `typeIdList` from
//! `icon_type_link`), keyed by the MD5 of the compressed bytes.
//!
//! Cached at the result level, so warm requests perform no database scan.

use anyhow::{Result, anyhow};
use sea_orm::{ColumnTrait, QueryFilter};
use serde::Serialize;

use _database::{
    DB_CONN, models::icon::icon as icon_model, models::icon::icon_type_link as itl_model,
};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{
    BinaryMd5Vo, CachedPage, ResultEntry, get_or_compute, get_result_cached, serialize_compress_md5,
};

/// camelCase icon view for the BinaryMD5 blob (Java `IconVo` naming).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconDocVo {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub type_id_list: Vec<i64>,
}

/// `GET /icon_doc/all_bin_md5` — MD5 of the single all-icons blob.
pub async fn do_all_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<BinaryMd5Vo>> {
    let entry = icon_result().await?;
    Ok(CommonResponse::new(Ok(entry.vo)))
}

/// `GET /icon_doc/all_bin` — the all-icons blob (compressed bytes).
pub async fn do_all_bin(_auth: AuthInfo) -> Result<Vec<u8>> {
    let entry = icon_result().await?;
    Ok(entry.bytes)
}

/// Compute (and cache) the single icon blob.
async fn icon_result() -> Result<ResultEntry> {
    let db = &DB_CONN.wait().pg_conn;

    let entries = get_result_cached("icon:result".into(), async {
        let icons = icon_model::Entity::find_safety().all(db).await?;
        let ids: Vec<i64> = icons.iter().map(|i| i.id).collect();

        // typeIdList per icon (Java: `IconTypeLink` by icon_id).
        let mut type_map: std::collections::BTreeMap<i64, Vec<i64>> =
            std::collections::BTreeMap::new();
        if !ids.is_empty() {
            for link in itl_model::Entity::find_safety()
                .filter(itl_model::Column::IconId.is_in(ids))
                .all(db)
                .await?
            {
                type_map.entry(link.icon_id).or_default().push(link.type_id);
            }
        }

        let vos: Vec<IconDocVo> = icons
            .into_iter()
            .map(|i| IconDocVo {
                id: i.id,
                name: i.tag,
                url: i.url,
                type_id_list: type_map.remove(&i.id).unwrap_or_default(),
            })
            .collect();
        let (compressed, md5_hex) = serialize_compress_md5(&vos)?;
        let page = get_or_compute("icon:all".to_string(), async {
            Ok(CachedPage {
                md5: md5_hex,
                time: chrono::Utc::now().timestamp_millis(),
                bytes: compressed,
            })
        })
        .await?;
        Ok(vec![ResultEntry {
            key: "icon:all".to_string(),
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
        .ok_or_else(|| anyhow!("empty icon result"))
}
