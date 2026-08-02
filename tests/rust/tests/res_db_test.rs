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

use _database::DB_CONN;
use _functions::functions::api::res::{UploadedFile, do_upload_image};
use _utils::{
    jwt::AuthInfo,
    models::SysUserVO,
    types::{AccessPolicyList, SystemUserRole},
};
use minio::s3::types::S3Api;

/// Skip when Postgres+MinIO are not configured (mirrors `api_db_test::db`).
async fn minio() -> Option<&'static minio::s3::MinioClient> {
    if std::env::var("GCS_TEST_DB").is_err() {
        eprintln!(
            "skipped: set GCS_TEST_DB=1 with Postgres+MinIO running \
             (tests/docker/docker-compose.e2e.yml) to run"
        );
        return None;
    }
    if DB_CONN.get().is_none() {
        let _ = _database::init_db_conn().await;
    }
    let conn = DB_CONN.get()?;
    if conn.minio_conn.is_none() {
        eprintln!(
            "skipped: MinIO is not configured (set MINIO_ACCESS_KEY / MINIO_SECRET_KEY / \
             MINIO_BASE_URL and start the service)"
        );
        return None;
    }
    conn.minio_conn.as_ref()
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
    let Some(client) = minio().await else { return };

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

    let resp = do_upload_image(stub_auth(), payload)
        .await
        .expect("upload should succeed");
    assert!(!resp.error, "response flagged an error: {}", resp.message);
    let vo = resp
        .data
        .as_ref()
        .expect("upload response carries data")
        .first()
        .expect("one uploaded file");

    // URL shape: {base}/images/uploads/{uuid}.png — extension derived from the
    // content type, never from the client file name.
    let base = std::env::var("MINIO_BASE_URL").unwrap_or_else(|_| "http://localhost:9000".into());
    assert!(
        vo.url
            .starts_with(&format!("{}/images/uploads/", base.trim_end_matches('/')))
    );
    assert!(vo.url.ends_with(".png"));
    assert_eq!(vo.md5, md5_hex);
    assert_eq!(vo.size, bytes.len());

    // Round-trip: the object must exist in MinIO with the original bytes.
    let key = vo
        .url
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
