use serde::{Deserialize, Serialize};

use crate::models::Pagination;

/// 图标基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconBaseRequest {
    /// 图标标签（Java `IconVo.tag` / 最新前端契约；旧前端用 `name`，
    /// serde alias 兼容历史调用方）。
    #[serde(alias = "name")]
    pub tag: String,
    /// 图标 URL（Java `IconVo.url` 可缺省；缺省落空串，避免 422）
    #[serde(default)]
    pub url: String,
    /// 图标分类 ID 列表（Java IconDto.typeIdList / 前端契约）：写入
    /// icon_type_link；此前 Rust 侧缺失该字段，前端归类被静默丢弃。
    #[serde(default)]
    pub type_id_list: Option<Vec<i64>>,
    /// 多分辨率 URL 变体（透传存储 `url_variants` jsonb 列）
    #[serde(default)]
    pub url_variants: Option<serde_json::Value>,
    /// 图标描述（透传存储 `description` 列）
    #[serde(default)]
    pub description: Option<String>,
}

/// 新增图标请求
pub type IconAddRequest = IconBaseRequest;

/// 更新图标请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconUpdateRequest {
    /// 图标 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础图标信息
    #[serde(flatten)]
    pub base: IconBaseRequest,
}

/// 图标列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconListRequest {
    /// 上传者
    pub creator: Option<i64>,
    /// 图标 ID 列表（旧字段，兼容保留）
    pub icon_list: Option<Vec<i64>>,
    /// 图标 ID 列表（前端契约 iconIdList）
    pub icon_id_list: Option<Vec<i64>>,
    /// 图标分类 ID 列表（前端契约 typeIdList，按 icon_type_link 过滤）
    pub type_id_list: Option<Vec<i64>>,
    /// 图标标签（最新前端 `tag`；旧前端 `name`，alias 兼容）
    #[serde(alias = "name")]
    pub tag: Option<String>,

    #[serde(flatten)]
    pub page: Pagination,
}

/// 图标返回值（用于 API 响应）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconVO {
    pub id: i64,
    /// 乐观锁版本号（前端编辑时随行提交，用于乐观锁校验）
    pub version: i64,
    pub creator_id: Option<i64>,
    pub create_time: f64,
    pub updater_id: Option<i64>,
    pub update_time: Option<f64>,
    /// 图标标签（Java `IconVo.tag`；旧 Rust 实现误用 `name`，最新前端
    /// 与 Java 均为 `tag`）
    pub tag: String,
    /// 图标分类 ID 列表（icon_type_link 反查）
    #[serde(default)]
    pub type_id_list: Vec<i64>,
    pub url: String,
    /// 多分辨率 URL 变体（jsonb 列透传；缺省为 null）
    pub url_variants: Option<serde_json::Value>,
    /// 图标描述（无描述为空串）
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconListResponse {
    pub total: i64,
    /// 每页大小（Java `PageListVo.size` 契约，默认 10）
    pub size: i64,
    #[serde(rename = "record")]
    pub items: Vec<IconVO>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconSingleResponse {
    pub item: IconVO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconAddResponse {
    pub id: i64,
}
