use serde::{Deserialize, Serialize};

use crate::types::HiddenFlag;

/// 点位基础请求模型
/// 去除了数据库模型中的ID、乐观锁、创建者/更新者信息和时间戳字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerRequest {
    /// 点位名称
    pub marker_title: Option<String>,
    /// 点位坐标
    /// 形如 "{x},{y}" 的格式，其中 x 与 y 均为浮点数文本
    pub position: String,
    /// 点位说明
    pub content: String,
    /// 点位图片
    pub picture: Option<String>,
    /// 点位初始标记者
    pub marker_creator_id: u64,
    /// 点位图片上传者
    pub picture_creator_id: Option<u64>,
    /// 点位视频
    pub video_path: Option<String>,
    /// 点位刷新时间
    /// 单位为毫秒
    pub refresh_time: u64,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 额外特殊字段
    pub extra: Option<serde_json::Value>,
}

/// 点位添加请求模型
pub type MarkerAddRequest = MarkerRequest;

/// 点位更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerUpdateRequest {
    /// 点位ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
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
    pub marker_creator_id: Option<u64>,
    /// 权限屏蔽标记
    pub hidden_flag: Option<HiddenFlag>,
    /// 分页页码
    pub page: Option<u32>,
    /// 分页大小
    pub page_size: Option<u32>,
}
