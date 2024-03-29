use anyhow::Result;
use axum::{routing::get, Router};

/// 公用物品 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/all_bz2",
            get(|| async move {
                // 查询所有物品信息，以 BZ2 压缩二进制数据后返回
                ""
            }),
        )
        .route(
            "/all_bz2_md5",
            get(|| async move {
                // 获取打包的 BZ2 压缩数据包的 MD5 值
                ""
            }),
        );

    Ok(router)
}
