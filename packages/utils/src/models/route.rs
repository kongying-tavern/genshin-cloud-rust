use crate::models::wrapper::Pagination;
use crate::types::HiddenFlag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 路线添加请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteAddRequest {
    /// 路线描述
    pub content: Option<String>,
    /// 创建人昵称
    pub creator_nickname: Option<String>,
    /// 额外信息
    pub extra: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 点位顺序数组
    pub marker_list: Vec<String>,
    /// 路线名称
    pub name: String,
    /// 视频地址
    pub video: Option<String>,
}

/// 路线更新请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteUpdateRequest {
    /// 路线描述
    pub content: Option<String>,
    /// 创建人昵称
    pub creator_nickname: Option<String>,
    /// 额外信息
    pub extra: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 路线 ID
    pub id: i64,
    /// 点位顺序数组
    pub marker_list: Vec<String>,
    /// 路线名称
    pub name: String,
    /// 视频地址
    pub video: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSearchRequest {
    /// 创建人 id，创建人 id，此字段不能与昵称模糊搜索字段共存
    pub creator_id: Option<String>,
    /// 创建人昵称模糊搜索字段，创建人昵称模糊搜索字段，此字段不能与创建人 id 字段共存
    pub creator_nickname_part: Option<String>,
    /// 路线名称模糊搜索字段
    pub name_part: Option<String>,
    /// 分页参数
    #[serde(flatten)]
    pub page: Pagination,
}

/// 路线视图对象
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteVO {
    /// 路线 ID
    pub id: i64,
    /// 路线名称
    pub name: String,
    /// 路线描述
    pub content: Option<String>,
    /// 点位顺序数组
    pub marker_list: Vec<i64>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 视频地址
    pub video: Option<String>,
    /// 额外信息
    pub extra: serde_json::Value,
    /// 创建人昵称
    pub creator_nickname: String,
    /// 创建人 ID
    pub creator_id: Option<i64>,
    /// 创建时间
    pub create_time: chrono::NaiveDateTime,
}

/// 路线分页返回
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePageResponse {
    /// 总条数
    pub total: i64,
    /// 当前页数据
    #[serde(rename = "record")]
    pub items: Vec<RouteVO>,
}
