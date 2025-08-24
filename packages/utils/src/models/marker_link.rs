use serde::{Deserialize, Serialize};

use crate::types::MarkerLinkageLinkAction;

/// 通用的组 ID 查询请求模型
/// 用于需要通过组 ID 列表进行查询的场景
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupIdsRequest {
    /// 关联组 ID 列表
    pub group_ids: Vec<String>,
}

/// 箭头类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrowType {
    #[serde(rename = "ARROW")]
    Arrow,
    #[serde(rename = "CIRCLE")]
    Circle,
    #[serde(rename = "DOT")]
    Dot,
    #[serde(rename = "NONE")]
    None,
}

/// 线条样式枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    #[serde(rename = "DASHED")]
    Dashed,
    #[serde(rename = "SOLID")]
    Solid,
}

/// 点位关联路径边
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerLinkagePathEdge {
    /// 起点箭头形状
    pub arrow_type1: Option<ArrowType>,
    /// 终点箭头形状
    pub arrow_type2: Option<ArrowType>,
    /// 起始曲线句柄 X 坐标，起始位置的三次贝塞尔曲线句柄 X 坐标
    pub handle_x1: Option<f64>,
    /// 终止曲线句柄 X 坐标，终止位置的三次贝塞尔曲线句柄 X 坐标
    pub handle_x2: Option<f64>,
    /// 起始曲线句柄 Y 坐标，起始位置的三次贝塞尔曲线句柄 Y 坐标
    pub handle_y1: Option<f64>,
    /// 终止曲线句柄 Y 坐标，终止位置的三次贝塞尔曲线句柄 Y 坐标
    pub handle_y2: Option<f64>,
    /// 起始点位 ID，输出时会转换为 X1 & Y1
    pub id1: Option<i64>,
    /// 终止点位 ID，输出时会转换为 X2 & Y2
    pub id2: Option<i64>,
    /// 线条样式
    pub line_type: Option<LineType>,
    /// 起始位置 X 坐标
    pub x1: Option<f64>,
    /// 终止位置 X 坐标
    pub x2: Option<f64>,
    /// 起始位置 Y 坐标
    pub y1: Option<f64>,
    /// 终止位置 Y 坐标
    pub y2: Option<f64>,
}

/// 点位关联
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerLinkage {
    /// 起始点点位 ID
    pub from_id: i64,
    /// 关联 ID
    pub id: i64,
    /// 关联关系
    pub link_action: Option<MarkerLinkageLinkAction>,
    /// 路线
    pub path: Option<Vec<Option<MarkerLinkagePathEdge>>>,
    /// 终止点点位 ID
    pub to_id: i64,
}

/// 点位关联列表查询请求
pub type MarkerLinkListRequest = GroupIdsRequest;

/// 点位关联图数据查询请求
pub type MarkerLinkGraphRequest = GroupIdsRequest;

/// 删除点位关联请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerLinkDeleteRequest {
    /// 关联组 ID
    pub group_ids: Option<Vec<String>>,
    /// 关联 ID
    pub ids: Option<Vec<i64>>,
}
