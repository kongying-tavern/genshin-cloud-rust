mod add;
mod create;
mod delete;
mod delete_by_name;
mod get_single;
mod list;
mod update;
mod update_by_name;
mod update_type;

use anyhow::Result;

use axum::{
    Router,
    routing::{delete as route_delete, post, put},
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::list))
        .route("/get/single/{name}", post(get_single::get_single))
        .route("/add", put(add::add))
        .route("/update", post(update::update))
        .route("/updateType", post(update_type::update_type))
        .route("/delete/{tagId}", route_delete(delete::delete))
        // 前端兼容路由（对齐 Java TagController 的路径契约）
        .route("/{tagName}", put(create::create))
        .route("/{tagName}", route_delete(delete_by_name::delete))
        .route("/{tagName}/{iconId}", post(update_by_name::update));

    Ok(ret)
}
