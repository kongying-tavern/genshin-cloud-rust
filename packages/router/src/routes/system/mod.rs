mod action_log;
mod archive;
mod device;
mod invitation;
pub mod oauth;
mod role;
mod user;

use anyhow::Result;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/archive/last/{}", get(archive::get_last))
        .route("/archive/history/{}", get(archive::get_history))
        .route("/archive/all_history", get(archive::get_all_history))
        .route("/archive/{}/{}", put(archive::put))
        .route("/archive/save/{}", post(archive::save))
        .route("/archive/rename/{}/{}", post(archive::rename))
        .route("/archive/restore/{}", delete(archive::restore))
        .route("/archive/slot/{}", delete(archive::delete_slot))
        .route("/action_log/list", post(action_log::list))
        .route("/device/list", post(device::list))
        .route("/device/update", post(device::update))
        .route("/invitation/list", post(invitation::list))
        .route("/invitation/update", post(invitation::update))
        .route("/invitation/info", post(invitation::info))
        .route("/invitation/consume", post(invitation::consume))
        .route("/invitation/{}", delete(invitation::delete))
        .route("/role/list", get(role::list))
        .route("/user/register", post(user::register))
        .route("/user/register/qq", post(user::register_qq))
        .route("/user/info/{}", get(user::get_info))
        .route("/user/update", post(user::update))
        .route("/user/update_password", post(user::update_password))
        .route(
            "/user/update_password_by_admin",
            post(user::update_password_by_admin),
        )
        .route("/user/{}", delete(user::delete))
        .route("/user/info/list", post(user::list))
        .route("/user/kick_out/{}", delete(user::kick_out));

    Ok(ret)
}
