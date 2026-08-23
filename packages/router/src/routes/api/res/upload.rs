use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractPunctuate;
use _functions::functions::api::res::UploadedFile;

/// 允许上传的内容类型白名单。
const ALLOWED_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];
/// 单个上传字段的大小上限（16 MiB，与全局 DefaultBodyLimit 一致）。
const MAX_FIELD_BYTES: usize = 16 * 1024 * 1024;

/// 上传图片
#[tracing::instrument(skip(auth))]
pub async fn upload_image(
    ExtractPunctuate(auth): ExtractPunctuate,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    // 收集文件字节与元数据，交给 functions 层落盘 MinIO。
    // 注意：这里不写临时文件——无主临时文件会泄漏磁盘，且字节最终
    // 需要原样上传。
    let mut files: Vec<UploadedFile> = Vec::new();
    let mut file_path: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| crate::routes::route_error(format!("multipart read error: {e}")))?
    {
        let name = field.name().map(|s| s.to_string()).unwrap_or_default();
        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_default();
        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();

        // 无文件名的字段视为文本字段（如 filePath），直接读取并跳过白名单校验。
        if file_name.is_empty() {
            let text = field.text().await.map_err(|e| {
                crate::routes::route_error(format!("multipart read text error: {e}"))
            })?;
            if name == "filePath" && file_path.is_none() {
                file_path = Some(text);
            }
            continue;
        }

        // 文件字段：内容类型白名单（防任意文件上传）+ 大小上限。
        if !ALLOWED_IMAGE_TYPES.contains(&content_type.as_str()) {
            return Err(crate::routes::route_error(format!(
                "unsupported content type '{content_type}' — allowed: {ALLOWED_IMAGE_TYPES:?}"
            )));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| crate::routes::route_error(format!("multipart read bytes error: {e}")))?;
        // 单字段大小上限（防内存/对象存储耗尽）。
        if data.len() > MAX_FIELD_BYTES {
            return Err(crate::routes::route_error(format!(
                "file exceeds the {MAX_FIELD_BYTES}-byte limit"
            )));
        }

        let size = data.len();
        let digest = md5::compute(&data);
        let md5_hex = format!("{:x}", digest);

        files.push(UploadedFile {
            field_name: name,
            original_file_name: file_name,
            content_type,
            size,
            md5: md5_hex,
            bytes: data.to_vec(),
        });
    }

    // 交给 functions 层（MinIO 未配置时明确报错，而非静默丢弃）。
    match _functions::functions::api::res::do_upload_image(auth, files, file_path).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
