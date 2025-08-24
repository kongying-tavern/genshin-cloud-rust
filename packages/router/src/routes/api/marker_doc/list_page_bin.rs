use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 点位分页数据
/// 查询分页点位信息，返回bz2压缩格式的byte数组
/// GET /marker_doc/list_page_bin/{md5}
#[tracing::instrument(skip_all)]
pub async fn list_page_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(md5): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据md5获取点位分页数据的逻辑
    Ok(())
}
