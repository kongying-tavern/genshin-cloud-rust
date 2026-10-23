use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAdmin;

/// 删除图标缓存（body 为图标 ID 列表，空列表 = 全量清除）
/// DELETE /cache/icon
///
/// 前端契约：map_register_v3 4.6.24 起 `cleanIconCache(number[])` 由
/// `/api/cache/iconTag` 迁移至此（对齐 Java `CacheController`）；与
/// iconTag（按标签名清理）是**不同语义**的两个端点，两者并存。
#[tracing::instrument(skip(db, auth))]
pub async fn delete_icon_cache(
    State(db): State<crate::routes::SharedDb>,
    ExtractAdmin(auth): ExtractAdmin,
    Json(icon_ids): Json<Vec<i64>>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::cache::do_delete_icon_cache(db.as_ref(), auth, icon_ids).await
    {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
