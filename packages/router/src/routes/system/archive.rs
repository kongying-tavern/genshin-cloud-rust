use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 获取指定槽位的最新存档
/// GET /archive/last/{slot_index}
#[tracing::instrument(skip(auth))]
pub async fn get_last(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_last(auth, user_id, slot_index as i32)
        .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 获取指定槽位的所有历史存档
/// GET /archive/history/{slot_index}
#[tracing::instrument(skip(auth))]
pub async fn get_history(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_history(auth, user_id, slot_index as i32)
        .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 获取所有槽位的历史存档
/// GET /archive/all_history
#[tracing::instrument(skip(auth))]
pub async fn get_all_history(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_all_history(auth, user_id).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 新建存档槽位并将存档存入
/// PUT /archive/{slot_index}/{name}
/// 请求体为任意 JSON（前端直接上传存档 JSON 文本；兼容 `{time, archive, historyIndex}` 包装体）
#[tracing::instrument(skip(auth))]
pub async fn put(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, name)): Path<(i64, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_save(
        auth,
        user_id,
        slot_index as i32,
        Some(name),
        payload,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 存档入指定槽位
/// POST /archive/save/{slot_index}
#[tracing::instrument(skip(auth))]
pub async fn save(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_save(
        auth,
        user_id,
        slot_index as i32,
        None,
        payload,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 重命名指定槽位
/// POST /archive/rename/{slot_index}/{new_name}
#[tracing::instrument(skip(auth))]
pub async fn rename(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, new_name)): Path<(i64, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_rename_by_slot(
        user_id,
        slot_index as i32,
        new_name,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 恢复为上次存档（删除最新一条，返回剩余最新一条存档）
/// DELETE /archive/restore/{slot_index}
#[tracing::instrument(skip(auth))]
pub async fn restore(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_restore_slot(user_id, slot_index as i32).await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 删除存档槽位
/// DELETE /archive/slot/{slot_index}
#[tracing::instrument(skip(auth))]
pub async fn delete_slot(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_delete_slot(user_id, slot_index as i32).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
