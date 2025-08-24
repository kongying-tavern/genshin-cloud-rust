use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::types::HiddenFlag;

/// 路线添加请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteAddRequest {
    /// 路线描述
    pub content: Option<String>,
    /// 创建人昵称
    pub creator_nickname: Option<String>,
    /// 额外信息
    pub extra: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 点位顺序数组
    pub marker_list: Vec<String>,
    /// 路线名称
    pub name: String,
    /// 视频地址
    pub video: Option<String>,
}

/// 路线更新请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteUpdateRequest {
    /// 路线描述
    pub content: Option<String>,
    /// 创建人昵称
    pub creator_nickname: Option<String>,
    /// 额外信息
    pub extra: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 路线 ID
    pub id: i64,
    /// 点位顺序数组
    pub marker_list: Vec<String>,
    /// 路线名称
    pub name: String,
    /// 视频地址
    pub video: Option<String>,
}

/// 新增路线
/// PUT /route/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现新增路线的逻辑
    Ok(())
}

/// 修改路线
/// POST /route
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现修改路线的逻辑
    Ok(())
}
