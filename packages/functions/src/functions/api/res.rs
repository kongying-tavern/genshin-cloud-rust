//! Resource (image) upload endpoints.
//!
//! Images are stored in MinIO (`images` bucket, `uploads/` prefix) and served
//! back as public URLs — the bucket is provisioned with an anonymous-read
//! policy at startup (see `build_minio_conn` in the database crate).

use anyhow::{Context, Result, anyhow};

use _database::DB_CONN;
use _utils::{jwt::AuthInfo, models::wrapper::CommonResponse};
use serde::{Deserialize, Serialize};

/// A file that passed the router's content-type/size gates and is ready to
/// be stored in MinIO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedFile {
    pub field_name: String,
    pub original_file_name: String,
    pub content_type: String,
    pub size: usize,
    pub md5: String,
    pub bytes: Vec<u8>,
}

/// `GET /res/get` 桩（**死代码**，保留仅为不破坏路由面）：
///
/// - Java 侧（`ResourceController`）只有 `PUT /upload/image`，无对应 GET 端点；
/// - 前端（`vue_map_register_v3`）只调用 `uploadImage`（`/api/res/upload/image`），
///   无 `res/get` 调用方。
///
/// 若未来协议侧定义该端点（如返回资源服务健康/配置信息），再按契约实现；
/// 当前恒返回空成功，不产生任何副作用。
pub async fn do_get() -> Result<CommonResponse<()>> {
    Ok(CommonResponse::new(Ok(())))
}

/// Extension for an upload, derived from the *content type* (never from the
/// client-supplied file name) so stored keys can't smuggle odd extensions.
fn ext_for_content_type(content_type: &str) -> &'static str {
    match content_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    }
}

/// Verify the file-header magic number matches the declared content type.
/// The content type is fully client-controlled, so MIME alone can't stop a
/// fake `image/png` HTML/SVG from being stored and served publicly.
/// Only image types are validated; other types pass through.
fn magic_matches_content_type(content_type: &str, bytes: &[u8]) -> bool {
    match content_type {
        // PNG: \x89PNG
        "image/png" => bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        // JPEG: \xFF\xD8
        "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8]),
        // GIF: GIF8
        "image/gif" => bytes.starts_with(b"GIF8"),
        // WEBP: RIFF....WEBP
        "image/webp" => bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP",
        _ => true,
    }
}

/// Store uploaded images in MinIO and return their public URLs.
///
/// Returns `{"filePath": ..., "fileUrl": ...}` for the first uploaded file
/// (对齐前端 `ResourceUploadVo` 契约：`filePath` 透传请求里的文本字段，
/// 缺省时回退为 `fileUrl`）。
///
/// Fails explicitly when MinIO is not configured — a silent success would
/// drop the upload on the floor and return a URL that doesn't exist.
pub async fn do_upload_image(
    auth: AuthInfo,
    payload: Vec<UploadedFile>,
    file_path: Option<String>,
) -> Result<CommonResponse<serde_json::Value>> {
    auth.require_non_anonymous()?;

    let client = DB_CONN.wait().minio_conn.clone().ok_or_else(|| {
        anyhow!("MinIO is not configured (set MINIO_BASE_URL, MINIO_ACCESS_KEY, MINIO_SECRET_KEY)")
    })?;
    let base_url =
        std::env::var("MINIO_BASE_URL").unwrap_or_else(|_| "http://localhost:9000".into());

    let f = payload
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no file uploaded"))?;

    if !magic_matches_content_type(&f.content_type, &f.bytes) {
        return Err(anyhow!(
            "uploaded file content does not match declared content type: {}",
            f.content_type
        ));
    }

    let key = format!(
        "uploads/{}.{}",
        uuid::Uuid::new_v4(),
        ext_for_content_type(&f.content_type)
    );
    client
        .put_object_content("images", &key, f.bytes)
        .map_err(|e| anyhow!("upload build failed: {e}"))?
        .content_type(f.content_type.clone())
        .build()
        .send()
        .await
        .context("upload to MinIO failed")?;

    let file_url = format!("{}/images/{}", base_url.trim_end_matches('/'), key);
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "filePath": file_path.unwrap_or_else(|| file_url.clone()),
        "fileUrl": file_url,
    }))))
}
