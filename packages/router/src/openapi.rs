//! OpenAPI 3.1 文档生成与 DEV 路由（utoipa）。
//!
//! 设计要点：
//! - 文档由 #[utoipa::path] 注解在**每个 handler 上**静态生成（编译期零运行时
//!   开销），聚合在 ApiDoc 中；
//! - 仅在 DEBUG 环境变量开启时挂载 /openapi.json 与 Swagger UI
//!   （/swagger-ui），并在启动时把文档落盘为 ./openapi.json。默认关闭，
//!   生产环境完全不暴露这些路由；
//! - 路径以 Java 契约的 /api/* 前缀为规范（dev 代理会把 /api/* 重写为
//!   /*，因此无前缀路径也能访问，但文档只列规范形式）；
//! - 绝大多数响应体为 CommonResponse<T> 信封（error/errorStatus/errorData/
//!   message/data/users/time），文档里用 inline(CommonResponse<X>) 展开，
//!   避免泛型实例在 components 中同名互踩。

use std::path::Path;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// 聚合全部路由注解的 OpenAPI 文档。
///
/// paths(...) 必须与 routes/mod.rs 里的路由表保持一一对应；漏挂的路由不会
/// 出现在文档里（openapi 测试对关键路径做了断言兜底）。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Genshin Map Cloud API",
        version = env!("CARGO_PKG_VERSION"),
        description = "空荧酒馆（Genshin Map Cloud）Rust 后端 API。\
\n\n- 鉴权：受保护接口需要在 Authorization 头携带 JWT（Bearer）；\
\n- 响应信封：绝大多数接口返回 CommonResponse<T>（error / errorStatus / errorData / message / data / users / time）；\
\n- 路径规范：领域接口以 /api/* 为前缀（Java 契约），dev 环境经 Vite 代理重写后无前缀路径同样可用；\
\n- 该文档仅在 DEBUG 环境变量开启时对外提供（/openapi.json 与 /swagger-ui）。",
    ),
    tags(
        (name = "app", description = "应用级操作（缓存刷新触发等）"),
        (name = "area", description = "地区域"),
        (name = "cache", description = "缓存管理（按域清理）"),
        (name = "history", description = "操作历史"),
        (name = "icon", description = "图标"),
        (name = "icon-doc", description = "图标数据文档（BinaryMD5 归档）"),
        (name = "icon-type", description = "图标分类"),
        (name = "item", description = "物品"),
        (name = "item-common", description = "地区公用物品"),
        (name = "item-doc", description = "物品数据文档（BinaryMD5 归档）"),
        (name = "item-type", description = "物品类型"),
        (name = "marker", description = "点位"),
        (name = "marker-doc", description = "点位数据文档（BinaryMD5 归档）"),
        (name = "marker-link", description = "点位关联"),
        (name = "marker-link-doc", description = "点位关联数据文档（BinaryMD5 归档）"),
        (name = "notice", description = "公告"),
        (name = "res", description = "资源（图片上传等）"),
        (name = "score", description = "评分统计"),
        (name = "tag", description = "标签"),
        (name = "tag-doc", description = "标签数据文档（BinaryMD5 归档）"),
        (name = "tag-type", description = "标签类型"),
        (name = "system", description = "系统管理（用户 / 设备 / 邀请 / 存档 / 日志 / 角色）"),
        (name = "auth", description = "认证（OAuth2 令牌 / JWKS）"),
        (name = "ws", description = "WebSocket"),
    ),
    paths(
        crate::routes::system::oauth::oauth,
        crate::routes::jwks,
        crate::routes::ws::ws_handler,
        crate::routes::ws::ws_handler_query,
        crate::routes::system::invitation::consume,
        crate::routes::system::invitation::list,
        crate::routes::system::invitation::update,
        crate::routes::system::invitation::info,
        crate::routes::system::invitation::delete,
        crate::routes::system::user::register,
        crate::routes::system::user::register_qq,
        crate::routes::system::user::get_info,
        crate::routes::system::user::update,
        crate::routes::system::user::update_password,
        crate::routes::system::user::update_password_by_admin,
        crate::routes::system::user::delete,
        crate::routes::system::user::list,
        crate::routes::system::user::kick_out,
        crate::routes::system::archive::get_last,
        crate::routes::system::archive::get_history,
        crate::routes::system::archive::get_all_history,
        crate::routes::system::archive::put,
        crate::routes::system::archive::save,
        crate::routes::system::archive::rename,
        crate::routes::system::archive::restore,
        crate::routes::system::archive::delete_slot,
        crate::routes::system::action_log::list,
        crate::routes::system::device::list,
        crate::routes::system::device::update,
        crate::routes::system::role::list,
        crate::routes::api::app::trigger_update,
        crate::routes::api::area::add::add,
        crate::routes::api::area::get::get,
        crate::routes::api::area::list::list,
        crate::routes::api::area::update::update,
        crate::routes::api::area::delete::delete,
        crate::routes::api::cache::icon_tag::delete_icon_tag_cache,
        crate::routes::api::cache::area::delete_area_cache,
        crate::routes::api::cache::item::delete_item_cache,
        crate::routes::api::cache::common_item::delete_common_item_cache,
        crate::routes::api::cache::marker::delete_marker_cache,
        crate::routes::api::cache::marker_link::delete_marker_link_cache,
        crate::routes::api::cache::notice::delete_notice_cache,
        crate::routes::api::history::list::get_list,
        crate::routes::api::icon::add::add,
        crate::routes::api::icon::delete::delete,
        crate::routes::api::icon::get_single::get_single,
        crate::routes::api::icon::list::list,
        crate::routes::api::icon::update::update,
        crate::routes::api::icon_doc::all_bin::all_bin,
        crate::routes::api::icon_doc::all_bin_md5::all_bin_md5,
        crate::routes::api::icon_type::add::add,
        crate::routes::api::icon_type::delete::delete,
        crate::routes::api::icon_type::list::list,
        crate::routes::api::icon_type::update::update,
        crate::routes::api::item::add::add,
        crate::routes::api::item::copy::copy_to_area,
        crate::routes::api::item::delete::delete,
        crate::routes::api::item::get_by_id::get_list_by_id,
        crate::routes::api::item::join::join_type,
        crate::routes::api::item::list::get_list,
        crate::routes::api::item::update::update,
        crate::routes::api::item_common::add::add,
        crate::routes::api::item_common::delete::delete,
        crate::routes::api::item_common::list::get_list,
        crate::routes::api::item_doc::list_page_bin::list_page_bin,
        crate::routes::api::item_doc::list_page_md5::list_page_bin_md5,
        crate::routes::api::item_type::add::add,
        crate::routes::api::item_type::delete::delete,
        crate::routes::api::item_type::list::get_list,
        crate::routes::api::item_type::list::get_list_all,
        crate::routes::api::item_type::move_type::move_to_target,
        crate::routes::api::item_type::update::update,
        crate::routes::api::marker::delete::delete,
        crate::routes::api::marker::get::get_id,
        crate::routes::api::marker::get::get_list_by_info,
        crate::routes::api::marker::get::get_list_by_id,
        crate::routes::api::marker::get::get_page,
        crate::routes::api::marker::single::add_single,
        crate::routes::api::marker::single::update_single,
        crate::routes::api::marker::tweak::tweak,
        crate::routes::api::marker_doc::list_diff_snapshot::list_diff_snapshot,
        crate::routes::api::marker_doc::list_page_bin::list_page_bin,
        crate::routes::api::marker_doc::list_page_md5::list_page_bin_md5,
        crate::routes::api::marker_link::delete::delete,
        crate::routes::api::marker_link::get::get_list,
        crate::routes::api::marker_link::get::get_graph,
        crate::routes::api::marker_link::link::link,
        crate::routes::api::marker_link_doc::doc::all_bin,
        crate::routes::api::marker_link_doc::doc::all_bin_md5,
        crate::routes::api::marker_link_doc::doc::all_graph_bin,
        crate::routes::api::marker_link_doc::doc::all_graph_bin_md5,
        crate::routes::api::notice::add::add_notice,
        crate::routes::api::notice::delete::delete_notice,
        crate::routes::api::notice::list::get_notice_list,
        crate::routes::api::notice::update::update_notice,
        crate::routes::api::res::get::get,
        crate::routes::api::res::upload::upload_image,
        crate::routes::api::score::generate::generate_score,
        crate::routes::api::score::data::get_score_data,
        crate::routes::api::tag::add::add,
        crate::routes::api::tag::create::create,
        crate::routes::api::tag::delete::delete,
        crate::routes::api::tag::delete_by_name::delete,
        crate::routes::api::tag::get_single::get_single,
        crate::routes::api::tag::list::list,
        crate::routes::api::tag::update::update,
        crate::routes::api::tag::update_by_name::update,
        crate::routes::api::tag::update_type::update_type,
        crate::routes::api::tag_doc::all_bin::all_bin,
        crate::routes::api::tag_doc::all_bin_md5::all_bin_md5,
        crate::routes::api::tag_type::add::add,
        crate::routes::api::tag_type::delete::delete,
        crate::routes::api::tag_type::list::list,
        crate::routes::api::tag_type::update::update,
    )
)]
pub struct ApiDoc;

/// 生成 OpenAPI 文档。仅用于 DEBUG 模式——生产路径完全不触碰。
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// DEV 路由：/openapi.json（由 SwaggerUi 的 url 注册）+ Swagger UI。
/// 仅在 debug_enabled() 时由 routes::router() 合并挂载。
pub fn router() -> Router {
    SwaggerUi::new("/swagger-ui")
        .url("/openapi.json", openapi())
        .into()
}

/// DEBUG 环境变量是否开启（大小写不敏感，接受 1/true/yes/on）。
/// 默认关闭：未设置或无法解析时一律视为关闭，生产环境永不暴露文档路由。
pub fn debug_enabled() -> bool {
    debug_enabled_value(std::env::var("DEBUG").ok().as_deref())
}

/// debug_enabled 的纯函数实现（便于单测）。
fn debug_enabled_value(value: Option<&str>) -> bool {
    value.is_some_and(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

/// 把 OpenAPI 文档以 pretty JSON 落盘到 path（DEBUG 模式下启动时调用）。
/// 失败不阻断启动（由调用方打日志）。
pub fn write_json_file(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(&openapi())?;
    std::fs::write(path.as_ref(), json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[test]
    fn debug_flag_parsing() {
        assert!(!debug_enabled_value(None));
        assert!(!debug_enabled_value(Some("")));
        assert!(!debug_enabled_value(Some("0")));
        assert!(!debug_enabled_value(Some("false")));
        assert!(debug_enabled_value(Some("1")));
        assert!(debug_enabled_value(Some("TRUE")));
        assert!(debug_enabled_value(Some("on")));
        assert!(debug_enabled_value(Some(" yes ")));
    }

    #[test]
    fn spec_covers_all_domains() {
        let doc = ApiDoc::openapi();
        let paths = &doc.paths.paths;
        for expected in [
            "/oauth/token",
            "/.well-known/jwks.json",
            "/ws/{user_id}",
            "/ws",
            "/system/user/register",
            "/system/archive/last/{slot_index}",
            "/api/area/get/list",
            "/api/area/add",
            "/api/cache/area",
            "/api/icon/get/list",
            "/api/icon_doc/all_bin",
            "/api/item/get/list",
            "/api/item_doc/list_page_bin/{md5}",
            "/api/marker/get/page",
            "/api/marker/tweak",
            "/api/marker_doc/list_diff_snapshot",
            "/api/marker_link/get/graph",
            "/api/notice/get/list",
            "/api/res/upload/image",
            "/api/score/data",
            "/api/tag/get/list",
            "/api/tag_doc/all_bin",
            "/api/tag_type/get/list",
        ] {
            assert!(paths.contains_key(expected), "missing path {expected}");
        }
        // 生成结果必须是可序列化的合法 JSON。
        let json = serde_json::to_value(&doc).expect("openapi spec must serialize to JSON");

        // 所有 $ref 必须能解析到 components.schemas（防御泛型同名 schema 互相覆盖）。
        let schemas = json["components"]["schemas"].as_object().expect("schemas");
        fn check_refs(
            node: &serde_json::Value,
            schemas: &serde_json::Map<String, serde_json::Value>,
        ) {
            match node {
                serde_json::Value::Object(map) => {
                    if let Some(serde_json::Value::String(r)) = map.get("$ref")
                        && let Some(name) = r.strip_prefix("#/components/schemas/")
                    {
                        assert!(schemas.contains_key(name), "dangling $ref: {r}");
                    }
                    for v in map.values() {
                        check_refs(v, schemas);
                    }
                },
                serde_json::Value::Array(items) => {
                    for v in items {
                        check_refs(v, schemas);
                    }
                },
                _ => {},
            }
        }
        check_refs(&json, schemas);
    }

    #[tokio::test]
    async fn docs_routes_are_debug_gated() {
        // rustls 进程级 provider 由 main() 安装；测试进程需要自己装一次
        // （routes::router() 构建 reqwest CDN 客户端时需要）。
        let _ = rustls::crypto::ring::default_provider().install_default();

        // 默认（DEBUG 未设置）：文档路由不存在，落到全局 fallback（501）。
        // `set_var`/`remove_var` 在 Rust 2024 为 unsafe：本测试独占该变量。
        unsafe { std::env::remove_var("DEBUG") };
        let router = crate::routes::router().await.expect("build router");
        let res = router
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call router");
        assert_eq!(res.status(), StatusCode::NOT_IMPLEMENTED);

        // DEBUG=1：/openapi.json 返回合法 JSON。
        unsafe { std::env::set_var("DEBUG", "1") };
        let router = crate::routes::router().await.expect("build router");
        let res = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call router");
        assert_eq!(res.status(), StatusCode::OK);
        let body = res
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&body).expect("json body");
        assert_eq!(parsed["openapi"], "3.1.0");
        assert!(parsed["paths"].as_object().is_some_and(|p| p.len() > 50));

        // Swagger UI：/swagger-ui 重定向到 /swagger-ui/，首页返回 HTML。
        let res = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/swagger-ui/")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call router");
        assert_eq!(res.status(), StatusCode::OK);

        unsafe { std::env::remove_var("DEBUG") };
    }
}
