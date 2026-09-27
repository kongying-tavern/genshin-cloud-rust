use anyhow::Result;

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 槽位范围校验（route 层收口）：前端契约固定 5 个存档槽位（0..=4），
/// 超限直接返回 400。校验通过后 i64 原值透传 do_*，不做 `as i32` 截断。
fn check_slot_index(slot_index: i64) -> Result<(), crate::routes::RouteError> {
    if !(0..=4).contains(&slot_index) {
        return Err(crate::routes::route_error(
            "slot_index must be in range 0..=4",
        ));
    }
    Ok(())
}

/// 获取指定槽位的最新存档
/// GET /archive/last/{slot_index}
#[tracing::instrument(skip(db, auth))]
pub async fn get_last(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_last(
        db.as_ref(),
        auth,
        user_id,
        slot_index,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取指定槽位的所有历史存档
/// GET /archive/history/{slot_index}
#[tracing::instrument(skip(db, auth))]
pub async fn get_history(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_history(
        db.as_ref(),
        auth,
        user_id,
        slot_index,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取所有槽位的历史存档
/// GET /archive/all_history
#[tracing::instrument(skip(db, auth))]
pub async fn get_all_history(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_get_all_history(db.as_ref(), auth, user_id)
        .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 新建存档槽位并将存档存入
/// PUT /archive/{slot_index}/{name}
/// 请求体为任意 JSON（前端直接上传存档 JSON 文本；兼容 `{time, archive, historyIndex}` 包装体）
#[tracing::instrument(skip(db, auth))]
pub async fn put(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, name)): Path<(i64, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_save(
        db.as_ref(),
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
#[tracing::instrument(skip(db, auth))]
pub async fn save(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_save(
        db.as_ref(),
        auth,
        user_id,
        slot_index,
        None,
        payload,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 重命名指定槽位
/// POST /archive/rename/{slot_index}/{new_name}
#[tracing::instrument(skip(db, auth))]
pub async fn rename(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, new_name)): Path<(i64, String)>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_rename_by_slot(
        db.as_ref(),
        auth,
        user_id,
        slot_index,
        new_name,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 恢复为上次存档（删除最新一条，返回剩余最新一条存档）
/// DELETE /archive/restore/{slot_index}
#[tracing::instrument(skip(db, auth))]
pub async fn restore(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_restore_slot(
        db.as_ref(),
        auth,
        user_id,
        slot_index,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 删除存档槽位
/// DELETE /archive/slot/{slot_index}
#[tracing::instrument(skip(db, auth))]
pub async fn delete_slot(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    check_slot_index(slot_index)?;
    let user_id = auth.info.id;
    match _functions::functions::system::archive::do_delete_slot(
        db.as_ref(),
        auth,
        user_id,
        slot_index,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
