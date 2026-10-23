//! Resource (image) upload endpoints.
//!
//! Images are stored in MinIO (`images` bucket, `uploads/` prefix) and served
//! back as public URLs — the bucket is provisioned with an anonymous-read
//! policy at startup (see `build_minio_conn` in the database crate).
//!
//! Two MinIO addresses are involved: `MINIO_BASE_URL` is the *internal*
//! endpoint the backend talks to for uploads, while the URLs returned to
//! clients are built from `MINIO_PUBLIC_BASE_URL` (the CDN/public endpoint;
//! falls back to `MINIO_BASE_URL` when unset). The two differ whenever a
//! CDN or reverse proxy rewrites the address between the backend and the
//! public internet.

use anyhow::{Context, Result, anyhow};

use _database::DatabaseConnectionMap;
use _utils::{errors::DomainError, jwt::AuthInfo, models::wrapper::CommonResponse};
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

/// 客户端提供的对象键安全校验（Java 无此防御；Rust 侧加固）：
/// 允许 `2025-09-09/abcdefg.png` 形式的相对子路径（字母/数字/`._-/`），
/// 拒绝绝对路径、`..` 穿越、反斜杠、控制字符与空键。返回规范化键。
fn sanitize_object_key(raw: &str) -> Option<String> {
    let key = raw.trim().trim_start_matches('/');
    if key.is_empty() || key.len() > 512 {
        return None;
    }
    if key.contains("..") || key.contains('\\') || key.contains('\u{0}') {
        return None;
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/'))
    {
        return None;
    }
    Some(key.to_string())
}

/// 公开访问 URL 模板（Java `image.minio.static-url-template` 的直译：
/// 空模板回退 `[[entrypoint]]/[[bucket]]/[[path]]`）。
fn public_url(public_base: &str, bucket: &str, key: &str) -> String {
    format!("{}/{}/{}", public_base.trim_end_matches('/'), bucket, key)
}

/// `GET /res/get?filePath=...`（Java `ResourceService.getResource`）：
/// 查询对象存储中同路径资源是否已存在；存在则返回其公开访问 URL，
/// 不存在返回空对象（前端 `getOrUploadImage` 以 `data.fileUrl` 判定，
/// 空对象即「未命中 → 走上传」）。
///
/// 前端把该端点用作**去重上传**的第一跳：命中则直接复用 URL，未命中才
/// `PUT /res/upload/image`。请求失败时前端自动降级为直接上传，因此
/// MinIO 未配置/不可达按业务错误返回（HTTP 200 + R 包装）而非 500。
pub async fn do_get(
    db: &DatabaseConnectionMap,
    file_path: Option<String>,
) -> Result<CommonResponse<serde_json::Value>> {
    let Some(raw_path) = file_path else {
        return Ok(CommonResponse::new(Ok(serde_json::json!({}))));
    };
    let Some(key) = sanitize_object_key(&raw_path) else {
        // 非法路径按「不存在」处理（Java 无此分支；Rust 侧路径校验的
        // 只读镜像——不把非法键回显给客户端）。
        return Ok(CommonResponse::new(Ok(serde_json::json!({}))));
    };
    let client = db.minio_conn.clone().ok_or_else(|| {
        DomainError::Business(
            "MinIO is not configured (set MINIO_BASE_URL, MINIO_ACCESS_KEY, MINIO_SECRET_KEY)"
                .into(),
        )
    })?;
    let public_base = std::env::var("MINIO_PUBLIC_BASE_URL").unwrap_or_else(|_| {
        std::env::var("MINIO_BASE_URL").unwrap_or_else(|_| "http://localhost:9000".into())
    });

    // 对象不存在（S3 NoSuchKey / 404）→ 未命中；其他错误（网络/凭据）
    // 按业务错误上抛，前端降级为直接上传。
    let stat = client
        .stat_object("images", &key)
        .map_err(|e| anyhow!("stat build failed: {e}"))?;
    let exists = stat_build_send(stat).await?;

    let body = if exists {
        serde_json::json!({
            "filePath": key,
            "fileUrl": super::rewrite_image_url(&public_url(&public_base, "images", &key)),
        })
    } else {
        serde_json::json!({})
    };
    Ok(CommonResponse::new(Ok(body)))
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

/// `stat_object(...)?` 后的 builder 需要 S3Api trait 在作用域内才能
/// `build().send()`；两处调用共用的小封装，返回对象是否存在。
async fn stat_build_send(stat: minio::s3::builders::StatObjectBldr) -> anyhow::Result<bool> {
    use minio::s3::types::S3Api;
    match stat.build().send().await {
        Ok(_) => Ok(true),
        Err(e) => {
            let msg = format!("{e}").to_lowercase();
            if msg.contains("nosuchkey") || msg.contains("not found") || msg.contains("404") {
                Ok(false)
            } else {
                Err(DomainError::Business(format!("读取文件失败：{e}")).into())
            }
        },
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
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    payload: Vec<UploadedFile>,
    file_path: Option<String>,
) -> Result<CommonResponse<serde_json::Value>> {
    auth.require_non_anonymous()?;

    // 配置缺失属内部故障而非业务文案，保持 anyhow（router 层兜底原文返回）。
    // 注：旧关键字方案会把本文案（含 "minio"）收敛为「请求失败」；现仅含
    // 环境变量名、无密钥值，原文返回可接受且更利于运维定位。
    let client = db.minio_conn.clone().ok_or_else(|| {
        anyhow!("MinIO is not configured (set MINIO_BASE_URL, MINIO_ACCESS_KEY, MINIO_SECRET_KEY)")
    })?;
    // Public base for the URLs returned to clients. Distinct from the
    // internal `MINIO_BASE_URL` used for the upload itself: a CDN may
    // rewrite the address between the backend and the public internet.
    let public_base = std::env::var("MINIO_PUBLIC_BASE_URL").unwrap_or_else(|_| {
        std::env::var("MINIO_BASE_URL").unwrap_or_else(|_| "http://localhost:9000".into())
    });

    let f = payload
        .into_iter()
        .next()
        .ok_or_else(|| DomainError::Business("no file uploaded".into()))?;

    if !magic_matches_content_type(&f.content_type, &f.bytes) {
        return Err(DomainError::Business(format!(
            "uploaded file content does not match declared content type: {}",
            f.content_type
        ))
        .into());
    }

    // 对象键 = 客户端提供的 filePath（Java `uploadImage` 契约：同路径
    // 复用、重复上传显式报错）。这是前端 `getOrUploadImage` 去重语义的
    // 另一半——`GET /res/get` 未命中后按原路径落盘，下次同路径即命中。
    let raw_path = file_path.as_deref().map(str::trim).unwrap_or_default();
    if raw_path.is_empty() {
        return Err(DomainError::Business("文件路径不能为空".into()).into());
    }
    let Some(key) = sanitize_object_key(raw_path) else {
        return Err(DomainError::Business(
            "文件路径仅允许字母、数字与 . _ - / 组成的相对子路径".into(),
        )
        .into());
    };

    // 同路径已存在 → 显式拒绝（Java「当前文件已存在，请更换文件路径」）。
    // 前端依赖该报错兜底 `GET /res/get` 未命中后的并发/重试重复上传。
    let stat = client
        .stat_object("images", &key)
        .map_err(|e| anyhow!("stat build failed: {e}"))?;
    if stat_build_send(stat).await? {
        return Err(DomainError::Business("当前文件已存在，请更换文件路径".into()).into());
    }

    client
        .put_object_content("images", &key, f.bytes)
        .map_err(|e| anyhow!("upload build failed: {e}"))?
        .content_type(f.content_type.clone())
        .build()
        .send()
        .await
        .context("upload to MinIO failed")?;

    let file_url = super::rewrite_image_url(&public_url(&public_base, "images", &key));
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "filePath": key,
        "fileUrl": file_url,
    }))))
}
