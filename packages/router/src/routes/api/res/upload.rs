use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};
use std::io::Write;
use std::path::PathBuf;

use crate::middlewares::ExtractAuthInfo;

/// 允许上传的内容类型白名单。
const ALLOWED_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];
/// 单个上传字段的大小上限（16 MiB，与全局 DefaultBodyLimit 一致）。
const MAX_FIELD_BYTES: usize = 16 * 1024 * 1024;

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

        // 内容类型白名单：拒绝非图片（防任意文件上传）。
        if !ALLOWED_IMAGE_TYPES.contains(&content_type.as_str()) {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                format!(
                    "unsupported content type '{content_type}' — allowed: {ALLOWED_IMAGE_TYPES:?}"
                ),
            ));
        }

        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("multipart read bytes error: {}", e),
            )
        })?;
        // 单字段大小上限（防磁盘耗尽）。
        if data.len() > MAX_FIELD_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("file exceeds the {MAX_FIELD_BYTES}-byte limit"),
            ));
        }

        // generate a unique filename（不信任用户文件名——仅取其扩展名）
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

        // 不返回 filesystem_path：避免向客户端泄露服务器绝对路径。
        files_meta.push(serde_json::json!({
            "field_name": name,
            "original_file_name": file_name,
            "content_type": content_type,
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
