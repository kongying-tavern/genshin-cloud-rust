use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{HiddenFlag, MarkerPunctuateMethodType, MarkerPunctuateStatus};

/// 打点信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PunctuateData {
    /// 申请者 ID，申请者 ID，提交请求时会校验
    pub author: i64,
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
    /// 操作类型
    pub method_type: MarkerPunctuateMethodType,
    /// 原有点位 ID
    pub original_marker_id: Option<f64>,
    /// 点位图片
    pub picture: Option<String>,
    /// 点位图片上传者
    pub picture_creator_id: Option<i64>,
    /// 点位坐标
    pub position: String,
    /// 打点提交 ID
    pub punctuate_id: f64,
    /// 刷新时间，+ 正数是毫秒
    /// + 0是不刷新
    /// + -1是次日4点
    /// + -2是点重置可以刷新的点位
    pub refresh_time: Option<i64>,
    /// 状态
    pub status: MarkerPunctuateStatus,
    /// 点位视频
    pub video_path: Option<String>,
}

/// 打点审核筛选请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PunctuateAuditFilterRequest {
    /// 地区 ID 列表
    pub area_id_list: Option<Vec<i64>>,
    /// 提交者 ID 列表
    pub author_list: Option<Vec<i64>>,
    /// 物品 ID 列表
    pub item_id_list: Option<Vec<i64>>,
    /// 类型 ID 列表
    pub type_id_list: Option<Vec<i64>>,
}
