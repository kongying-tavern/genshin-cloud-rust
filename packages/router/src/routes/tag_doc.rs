use std::sync::Arc;

use anyhow::Result;
use axum::{routing::get, Extension, Router};

use crate::SharedDatabaseConnection;

/// 图标标签档案 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/all_bz2",
            get(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>| async move {
                    // 查询所有标签信息，以 BZ2 压缩二进制数据后返回
                    ""
                },
            ),
        )
        .route(
            "/all_bz2_md5",
            get(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>| async move {
                    // 获取打包的 BZ2 压缩数据包的 MD5 值
                    ""
                },
            ),
        );

    Ok(router)
}
