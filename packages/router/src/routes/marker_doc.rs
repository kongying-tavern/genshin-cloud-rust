use anyhow::Result;
use axum::{extract::Path, routing::get, Router};

/// 点位档案 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/list_page_bz2/:index",
            get(|Path(_index): Path<String>| async move {
                // 查询分页点位信息，以 BZ2 压缩二进制数据后返回
                ""
            }),
        )
        .route(
            "/list_page_bz2_md5",
            get(|| async move {
                // 获取打包的 BZ2 压缩数据包的 MD5 值
                ""
            }),
        );

    Ok(router)
}
