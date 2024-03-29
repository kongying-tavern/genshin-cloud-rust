use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Json, Router,
};

use _utils::schemas::{tag::Schema as TagSchema, tag_search::Schema as TagSearchSchema};

/// 图标 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(|Json(_frm): Json<TagSearchSchema>| async move {
                // 列出标签
                ""
            }),
        )
        .route(
            "/get/single/:name",
            post(|Path(_name): Path<String>| async move {
                // 获取单个标签信息
                ""
            }),
        )
        .route(
            "/:tag_name/:icon_id",
            post(
                |Path((_tag_name, _icon_id)): Path<(String, String)>| async move {
                    // 修改标签关联
                    ""
                },
            ),
        )
        .route(
            "/updateType",
            post(|Json(_frm): Json<TagSchema>| async move {
                // 修改标签的分类信息；该接口仅在后台使用
                ""
            }),
        )
        .route(
            "/:tag_name",
            put(|Path(_tag_name): Path<String>| async move {
                // 创建一个空标签
                ""
            }),
        )
        .route(
            "/:tag_name",
            delete(|Path(_tag_name): Path<String>| async move {
                // 删除标签；需要确保已经没有条目在使用这个标签，否则会删除失败
                ""
            }),
        );

    Ok(router)
}
