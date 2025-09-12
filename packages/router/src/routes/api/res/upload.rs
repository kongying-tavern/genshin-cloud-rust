use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};
use std::io::Write;
use std::path::PathBuf;

use crate::middlewares::ExtractAuthInfo;

/// 上传图片
#[tracing::instrument(skip(auth))]
pub async fn upload_image(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Save uploaded files to temp dir and collect metadata
    let tmp_dir = std::env::temp_dir();
    let mut files_meta: Vec<serde_json::Value> = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("multipart read error: {}", e),
        )
    })? {
        let name = field.name().map(|s| s.to_string()).unwrap_or_default();
        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_default();
        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();
        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("multipart read bytes error: {}", e),
            )
        })?;

        // generate a unique filename
        let uuid = uuid::Uuid::new_v4().to_string();
        let ext = PathBuf::from(&file_name)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "bin".to_string());
        let filename = format!("upload_{}_{}.{}", uuid, chrono::Utc::now().timestamp(), ext);
        let mut file_path = tmp_dir.clone();
        file_path.push(&filename);

        // write to file
        let mut f = std::fs::File::create(&file_path).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("create file error: {}", e),
            )
        })?;
        f.write_all(&data).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("write file error: {}", e),
            )
        })?;

        let size = data.len();
        let digest = md5::compute(&data);
        let md5_hex = format!("{:x}", digest);

        files_meta.push(serde_json::json!({
            "field_name": name,
            "original_file_name": file_name,
            "content_type": content_type,
            "filesystem_path": file_path.to_string_lossy().to_string(),
            "size": size,
            "md5": md5_hex,
        }));
    }

    // send metadata array to functions layer
    let payload = serde_json::Value::Array(files_meta);
    match _functions::functions::api::res::do_upload_image(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
