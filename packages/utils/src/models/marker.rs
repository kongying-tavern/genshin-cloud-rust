use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{models::wrapper::Pagination, types::HiddenFlag};
use serde_json::Value;

/// 点位物品关联（Java `MarkerItemLinkVo`；前端按 `itemList` 过滤/渲染点位，
/// 缺失会导致任何点位都无法显示）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerItemLinkVo {
    pub item_id: i64,
    pub count: i32,
    pub icon_tag: Option<String>,
    /// 图标 ID（远程 schema 列）
    pub icon_id: i64,
}

/// 点位对外返回 VO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerVO {
    pub version: i64,
    pub id: i64,
    pub create_time: f64,
    pub update_time: Option<f64>,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub del_flag: bool,

    /// 仅兼容历史数据，业务层仍需读取旧记录
    pub marker_stamp: Option<String>,
    pub marker_title: Option<String>,
    pub position: String,
    pub content: Option<String>,
    pub picture: Option<String>,
    pub marker_creator_id: i64,
    pub picture_creator_id: Option<i64>,
    pub video_path: Option<String>,
    pub refresh_time: i64,
    pub hidden_flag: HiddenFlag,
    /// 直接保留原始 extra JSON
    pub extra: Option<Value>,
    /// 点位物品关联（`marker_item_link`），前端过滤/图标依赖
    #[serde(default)]
    pub item_list: Vec<MarkerItemLinkVo>,
    /// 关联（marker_linkage）ID，前端按 linkageId 关联展示；
    /// 当前 marker 域接口未查询 marker_linkage，恒为 None
    #[serde(default)]
    pub linkage_id: Option<String>,
}

/// 空响应（会序列化为 {}）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerEmptyResponse {}

/// 添加返回 ID
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerAddResponse {
    pub id: i64,
}

/// ID 列表响应
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerIdListResponse {
    pub ids: Vec<i64>,
}

/// 列表响应（带总数）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerListResponse {
    pub total: usize,
    /// 分页大小（前端 PageListVoMarkerVo 的 size）
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(rename = "record")]
    pub items: Vec<MarkerVO>,
}

/// 仅 items 响应（不含 total）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerItemsResponse {
    #[serde(rename = "record")]
    pub items: Vec<MarkerVO>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerSingleResponse {
    pub marker: MarkerVO,
}

/// 点位基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerRequest {
    /// 点位名称
    pub marker_title: Option<String>,
    /// 点位坐标
    /// 形如 "{x},{y}" 的格式，其中 x 与 y 均为浮点数文本
    pub position: String,
    /// 点位说明
    pub content: Option<String>,
    /// 点位图片
    pub picture: Option<String>,
    /// 点位初始标记者
    pub marker_creator_id: i64,
    /// 点位图片上传者
    pub picture_creator_id: Option<i64>,
    /// 点位视频
    pub video_path: Option<String>,
    /// 点位刷新时间
    /// 单位为毫秒
    pub refresh_time: i64,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 额外特殊字段
    pub extra: Option<serde_json::Value>,
}

/// 点位筛选查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerFilterRequest {
    /// 地区 ID 列表
    pub area_id_list: Option<Vec<i64>>,
    /// 物品 ID 列表
    pub item_id_list: Option<Vec<i64>>,
    /// 类型 ID 列表
    pub type_id_list: Option<Vec<i64>>,
}

/// 点位添加请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerAddRequest {
    /// 点位说明
    pub content: Option<String>,
    /// 额外字段
    pub extra: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 点位物品 ID 列表
    pub item_list: Vec<Option<serde_json::Value>>,
    /// 点位初始标记者，仅创建时写入，后续不可更改，创建时会校验
    pub marker_creator_id: i64,
    /// 点位标题
    pub marker_title: String,
    /// 点位图片
    pub picture: Option<String>,
    /// 点位图片上传者
    pub picture_creator_id: Option<i64>,
    /// 点位坐标
    pub position: String,
    /// 刷新时间，+ 正数是毫秒
    /// + 0是不刷新
    /// + -1是次日4点
    /// + -2是点重置可以刷新的点位
    pub refresh_time: Option<i64>,
    /// 点位视频
    pub video_path: Option<String>,
}

/// 点位更新请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerUpdateData {
    /// 点位说明
    pub content: Option<String>,
    /// 乐观锁版本号（Java `@Version` 契约）：携带时按该版本做条件更新，
    /// 冲突返回「该点位已更新，请重新提交」；缺省时按库中当前版本更新。
    #[serde(default)]
    pub version: Option<i64>,
    /// 额外字段
    pub extra: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 点位 ID
    pub id: i64,
    /// 点位物品 ID 列表
    pub item_list: Vec<Option<serde_json::Value>>,
    /// 点位初始标记者，仅创建时写入，后续不可更改，创建时会校验
    pub marker_creator_id: i64,
    /// 点位标题
    pub marker_title: String,
    /// 点位图片
    pub picture: Option<String>,
    /// 点位图片上传者
    pub picture_creator_id: Option<i64>,
    /// 点位坐标
    pub position: String,
    /// 刷新时间，+ 正数是毫秒
    /// + 0是不刷新
    /// + -1是次日4点
    /// + -2是点重置可以刷新的点位
    pub refresh_time: Option<i64>,
    /// 点位视频
    pub video_path: Option<String>,
}

/// 点位调整配置枚举 - 属性类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkerTweakConfigPropEnum {
    Content,
    Extra,
    #[serde(rename = "hiddenFlag")]
    HiddenFlag,
    #[serde(rename = "itemList")]
    ItemList,
    Position,
    #[serde(rename = "refreshTime")]
    RefreshTime,
    Title,
    #[serde(rename = "videoPath")]
    VideoPath,
}

/// 点位调整配置枚举 - 调整类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkerTweakConfigTypeEnum {
    Append,
    #[serde(rename = "insertIfAbsent")]
    InsertIfAbsent,
    #[serde(rename = "insertOrUpdate")]
    InsertOrUpdate,
    Merge,
    Prepend,
    #[serde(rename = "removeLeft")]
    RemoveLeft,
    #[serde(rename = "removeRight")]
    RemoveRight,
    Replace,
    #[serde(rename = "replaceRegex")]
    ReplaceRegex,
    #[serde(rename = "trimLeft")]
    TrimLeft,
    #[serde(rename = "trimRight")]
    TrimRight,
    Update,
}

/// 数据值枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TweakValue {
    AnythingArray(Vec<Option<serde_json::Value>>),
    AnythingMap(HashMap<String, Option<serde_json::Value>>),
    Bool(bool),
    // Integer 须在 Double 前：untagged 下 JSON 数字先按 i64 解析，
    // 否则前端发送的整数值（refreshTime/hiddenFlag）全被解析为 Double 而静默失效
    Integer(i64),
    Double(f64),
    String(String),
}

/// 调整配置数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakMeta {
    /// 物品关联
    pub item_list: Option<Vec<Option<serde_json::Value>>>,
    /// 键值对映射
    pub map: Option<HashMap<String, Option<serde_json::Value>>>,
    /// 替换为
    pub replace: Option<String>,
    /// 检查文本
    pub test: Option<String>,
    /// 数据值
    pub value: Option<TweakValue>,
}

/// 点位调整配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerTweakConfig {
    /// 调整配置数据，此项根据调整方法类型使用不同的数据字段
    pub meta: TweakMeta,
    /// 需调整的点位属性
    pub prop: MarkerTweakConfigPropEnum,
    /// 调整方法类型
    #[serde(rename = "type")]
    pub marker_tweak_config_type: MarkerTweakConfigTypeEnum,
}

/// 点位调整请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerTweakRequest {
    /// 点位 ID 列表
    pub marker_ids: Vec<i64>,
    /// 点位数据调整配置
    pub tweaks: Vec<MarkerTweakConfig>,
}

/// 原有的点位更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerUpdateRequest {
    /// 点位 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础点位信息
    #[serde(flatten)]
    pub marker: MarkerRequest,
}

/// 点位列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerListRequest {
    /// 点位名称（模糊搜索）
    pub marker_title: Option<String>,
    /// 点位初始标记者
    pub marker_creator_id: Option<i64>,
    /// 权限屏蔽标记
    pub hidden_flag: Option<HiddenFlag>,

    #[serde(flatten)]
    pub page: Pagination,
}
