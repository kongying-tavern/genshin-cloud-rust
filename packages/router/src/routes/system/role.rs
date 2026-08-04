use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::SysRoleVo;

/// 角色列表（Java `SysRoleVo`；前端用 `code` 映射权限掩码）。
/// GET /role/list
#[tracing::instrument(skip(_auth))]
pub async fn list(
    ExtractAuthInfo(_auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let roles = vec![
        SysRoleVo {
            id: 0,
            name: "系统管理员".into(),
            code: "ADMIN".into(),
            sort: 2,
        },
        SysRoleVo {
            id: 1,
            name: "地图管理员".into(),
            code: "MAP_MANAGER".into(),
            sort: 3,
        },
        SysRoleVo {
            id: 2,
            name: "测试打点员".into(),
            code: "MAP_NEIGUI".into(),
            sort: 4,
        },
        SysRoleVo {
            id: 3,
            name: "地图打点员".into(),
            code: "MAP_PUNCTUATE".into(),
            sort: 5,
        },
        SysRoleVo {
            id: 4,
            name: "地图用户".into(),
            code: "MAP_USER".into(),
            sort: 6,
        },
        SysRoleVo {
            id: 5,
            name: "匿名用户".into(),
            code: "VISITOR".into(),
            sort: 100,
        },
    ];
    Ok(Json(roles).into_response())
}
