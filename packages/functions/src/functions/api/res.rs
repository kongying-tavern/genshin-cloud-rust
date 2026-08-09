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
