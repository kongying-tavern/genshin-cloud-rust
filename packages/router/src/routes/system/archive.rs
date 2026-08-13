use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo, api_error};
use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

/// 槽位范围校验（route 层收口）：前端契约固定 5 个存档槽位（0..=4），
/// 超限直接返回 400。校验通过后 i64 原值透传 do_*，不做 `as i32` 截断。
fn check_slot_index(slot_index: i64) -> Result<(), ApiError> {
    if !(0..=4).contains(&slot_index) {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "slot_index must be in range 0..=4",
        ));
    }
    Ok(())
}

/// 获取指定槽位的最新存档
/// GET /archive/last/{slot_index}
#[utoipa::path(
    get,
    path = "/system/archive/last/{slot_index}",
    tag = "system",
    summary = "获取指定槽位的最新存档",
    params(("slot_index" = i64, Path, description = "槽位下标（0..=4）")),
    responses(
        (status = 200, description = "最新存档", body = inline(CommonResponse<Option<_utils::models::ArchiveSlotVo>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_last(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_last(auth, user_id, slot_index).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取指定槽位的所有历史存档
/// GET /archive/history/{slot_index}
#[utoipa::path(
    get,
    path = "/system/archive/history/{slot_index}",
    tag = "system",
    summary = "获取指定槽位的所有历史存档",
    params(("slot_index" = i64, Path, description = "槽位下标（0..=4）")),
    responses(
        (status = 200, description = "历史存档列表（JSON 数组）", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_history(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_history(auth, user_id, slot_index).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取所有槽位的历史存档
/// GET /archive/all_history
#[utoipa::path(
    get,
    path = "/system/archive/all_history",
    tag = "system",
    summary = "获取所有槽位的历史存档",
    responses(
        (status = 200, description = "历史存档列表（JSON 数组）", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_all_history(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_all_history(auth, user_id).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 新建存档槽位并将存档存入
/// PUT /archive/{slot_index}/{name}
/// 请求体为任意 JSON（前端直接上传存档 JSON 文本；兼容 `{time, archive, historyIndex}` 包装体）
#[utoipa::path(
    put,
    path = "/system/archive/{slot_index}/{name}",
    tag = "system",
    summary = "新建存档槽位并将存档存入",
    params(
        ("slot_index" = i64, Path, description = "槽位下标（0..=4）"),
        ("name" = String, Path, description = "槽位名称"),
    ),
    request_body = inline(serde_json::Value),
    responses(
        (status = 200, description = "保存结果", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn put(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, name)): Path<(i64, String)>,
    AppJson(payload): AppJson<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_save(
        auth,
        user_id,
        slot_index,
        Some(name),
        payload,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 存档入指定槽位
/// POST /archive/save/{slot_index}
#[utoipa::path(
    post,
    path = "/system/archive/save/{slot_index}",
    tag = "system",
    summary = "存档入指定槽位",
    params(("slot_index" = i64, Path, description = "槽位下标（0..=4）")),
    request_body = inline(serde_json::Value),
    responses(
        (status = 200, description = "保存结果", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn save(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
    AppJson(payload): AppJson<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_save(auth, user_id, slot_index, None, payload)
        .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 重命名指定槽位
/// POST /archive/rename/{slot_index}/{new_name}
#[utoipa::path(
    post,
    path = "/system/archive/rename/{slot_index}/{new_name}",
    tag = "system",
    summary = "重命名指定槽位",
    params(
        ("slot_index" = i64, Path, description = "槽位下标（0..=4）"),
        ("new_name" = String, Path, description = "新槽位名称"),
    ),
    responses(
        (status = 200, description = "重命名结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn rename(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, new_name)): Path<(i64, String)>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_rename_by_slot(
        auth, user_id, slot_index, new_name,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 恢复为上次存档（删除最新一条，返回剩余最新一条存档）
/// DELETE /archive/restore/{slot_index}
#[utoipa::path(
    delete,
    path = "/system/archive/restore/{slot_index}",
    tag = "system",
    summary = "恢复为上次存档",
    params(("slot_index" = i64, Path, description = "槽位下标（0..=4）")),
    responses(
        (status = 200, description = "剩余的最新一条存档", body = inline(CommonResponse<Option<_utils::models::ArchiveSlotVo>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn restore(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_restore_slot(auth, user_id, slot_index).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 删除存档槽位
/// DELETE /archive/slot/{slot_index}
#[utoipa::path(
    delete,
    path = "/system/archive/slot/{slot_index}",
    tag = "system",
    summary = "删除存档槽位",
    params(("slot_index" = i64, Path, description = "槽位下标（0..=4）")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete_slot(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_delete_slot(auth, user_id, slot_index).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
