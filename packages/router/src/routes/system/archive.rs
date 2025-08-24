use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::types::SystemUserRole;

/// 获取指定槽位的最新存档
/// GET /archive/last/{slot_index}
#[tracing::instrument(skip_all)]
pub async fn get_last(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

/// 获取指定槽位的所有历史存档
/// GET /archive/history/{slot_index}
#[tracing::instrument(skip_all)]
pub async fn get_history(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

/// 获取所有槽位的历史存档
/// GET /archive/all_history
#[tracing::instrument(skip_all)]
pub async fn get_all_history(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSaveParams {
    pub time: DateTime<Local>,
    pub archive: String,
    pub history_index: u32,
}

/// 新建存档槽位并将存档存入
/// PUT /archive/{slot_index}/{name}
#[tracing::instrument(skip_all)]
pub async fn put(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, name)): Path<(u64, String)>,
    Json(payload): Json<ArchiveSaveParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

/// 存档入指定槽位
/// POST /archive/save/{slot_index}
#[tracing::instrument(skip_all)]
pub async fn save(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<u64>,
    Json(payload): Json<ArchiveSaveParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

/// 重命名指定槽位
/// POST /archive/rename/{slot_index}/{new_name}
#[tracing::instrument(skip_all)]
pub async fn rename(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((slot_index, new_name)): Path<(u64, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

/// 删除最近一次存档（恢复为上次存档）
/// DELETE /archive/restore/{slot_index}
#[tracing::instrument(skip_all)]
pub async fn restore(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

/// 删除存档槽位
/// DELETE /archive/slot/{slot_index}
#[tracing::instrument(skip_all)]
pub async fn delete_slot(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(slot_index): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}
