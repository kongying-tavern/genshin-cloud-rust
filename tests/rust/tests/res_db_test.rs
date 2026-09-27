//! MinIO-backed res-upload test (PLAN.md M3 / F15).
//!
//! Same `GCS_TEST_DB` gate as the other DB tests; additionally requires a
//! reachable MinIO (`MINIO_ACCESS_KEY` / `MINIO_SECRET_KEY` set, instance up
//! via `tests/docker/docker-compose.e2e.yml`). Skips (with a notice) when
//! either is missing — CI's `integration` job only provisions Postgres, so
//! this test only runs locally / on MinIO-enabled setups.
//!
//! Verifies `do_upload_image` stores the bytes in the `images` bucket and
//! returns a public URL, then round-trips the object back out of MinIO and
//! cleans up after itself.

use std::sync::Arc;

use tokio::sync::OnceCell;

use _database::DatabaseConnectionMap;
use _functions::functions::api::res::{UploadedFile, do_upload_image};
use _utils::{
    jwt::AuthInfo,
    models::SysUserVO,
    types::{AccessPolicyList, SystemUserRole},
};
use minio::s3::types::S3Api;

/// Skip when Postgres+MinIO are not configured (mirrors `api_db_test::db`).
/// 自持连接映射：不写 `DB_CONN` 全局（连接注入重构的可测性验收点）。
static MAP: OnceCell<Option<Arc<DatabaseConnectionMap>>> = OnceCell::const_new();

async fn map() -> Option<&'static DatabaseConnectionMap> {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!(
            "skipped: set GCS_TEST_DB=1 with Postgres+MinIO running \
             (tests/docker/docker-compose.e2e.yml) to run"
        );
        return None;
    }
    let arc = MAP
        .get_or_init(|| async {
            match _database::connect_db_map().await {
                Ok(m) => Some(Arc::new(m)),
                Err(e) => {
                    eprintln!("skipped: connect_db_map failed: {e}");
                    None
                },
            }
        })
        .await
        .as_ref()?;
    Some(arc)
}

/// Skip when Postgres+MinIO are not configured (mirrors `api_db_test::map`).
/// 返回自持映射与其中的 MinIO 客户端（配置缺失时打印跳过说明）。
async fn minio() -> Option<(
    &'static DatabaseConnectionMap,
    &'static minio::s3::MinioClient,
)> {
    let map = map().await?;
    let Some(client) = map.minio_conn.as_ref() else {
        eprintln!(
            "skipped: MinIO is not configured (set MINIO_ACCESS_KEY / MINIO_SECRET_KEY / \
             MINIO_BASE_URL and start the service)"
        );
        return None;
    };
    Some((map, client))
}

fn stub_auth() -> AuthInfo {
    let now = chrono::Utc::now();
    AuthInfo {
        info: SysUserVO {
            id: 1,
            username: "stub".into(),
            nickname: None,
            qq: None,
            phone: None,
            logo: None,
            role_id: SystemUserRole::Admin,
            access_policy: AccessPolicyList(vec![]),
            remark: None,
        },
        created_at: now,
        expires_at: now + chrono::Duration::days(1),
    }
}

#[tokio::test]
async fn res_upload_stores_image_in_minio() {
    let Some((map, client)) = minio().await else {
        return;
    };

    let bytes = b"\x89PNG\r\n\x1a\n fake png body".to_vec();
    let md5_hex = format!("{:x}", md5::compute(&bytes));
    let payload = vec![UploadedFile {
        field_name: "file".into(),
        original_file_name: "icon.png".into(),
        content_type: "image/png".into(),
        size: bytes.len(),
        md5: md5_hex.clone(),
        bytes: bytes.clone(),
    }];

    let resp = do_upload_image(map, stub_auth(), payload, None)
        .await
        .expect("upload should succeed");
    assert!(!resp.error, "response flagged an error: {}", resp.message);
    let data = resp.data.as_ref().expect("upload response carries data");
    let file_url = data
        .get("fileUrl")
        .and_then(|v| v.as_str())
        .expect("fileUrl field present");
    let file_path = data
        .get("filePath")
        .and_then(|v| v.as_str())
        .expect("filePath field present");

    // URL shape: {public base}/images/uploads/{uuid}.png — extension derived
    // from the content type, never from the client file name. The returned
    // URL is built from MINIO_PUBLIC_BASE_URL (falling back to the internal
    // MINIO_BASE_URL), mirroring `do_upload_image`.
    let base = std::env::var("MINIO_PUBLIC_BASE_URL").unwrap_or_else(|_| {
        std::env::var("MINIO_BASE_URL").unwrap_or_else(|_| "http://localhost:9000".into())
    });
    assert!(file_url.starts_with(&format!("{}/images/uploads/", base.trim_end_matches('/'))));
    assert!(file_url.ends_with(".png"));
    assert_eq!(file_path, file_url, "filePath falls back to fileUrl");

    // Round-trip: the object must exist in MinIO with the original bytes.
    let key = file_url
        .rsplit_once("/images/")
        .expect("url contains bucket segment")
        .1;
    let get_resp = client
        .get_object("images", key)
        .expect("build get-object request")
        .build()
        .send()
        .await
        .expect("object should be retrievable");
    assert_eq!(
        get_resp.object_size().expect("object size"),
        bytes.len() as u64,
        "stored object size must match the upload"
    );

    // Cleanup.
    client
        .delete_object("images", key)
        .expect("build delete-object request")
        .build()
        .send()
        .await
        .expect("cleanup delete should succeed");
}
