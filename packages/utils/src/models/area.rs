use serde::{Deserialize, Serialize};

use crate::types::HiddenFlag;

/// 地区基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaRequest {
    /// 地区名称
    pub name: String,
    /// 地区代码
    pub code: Option<String>,
    /// 地区说明
    pub content: Option<String>,
    /// 图标标签
    pub icon_tag: String,
    /// 父级地区 ID
    /// 无父级则为 -1
    pub parent_id: i64,
    /// 是否为末端地区
    pub is_final: bool,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 排序
    pub sort_index: i32,
    /// 额外标记
    /// 低位第一位：前台是否显示
    pub special_flag: i32,
}

/// 地区添加请求模型
/// 继承基础 AreaRequest，可以在此基础上添加特定于添加操作的字段
pub type AreaAddRequest = AreaRequest;

/// 地区更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaUpdateRequest {
    /// 地区 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础地区信息
    #[serde(flatten)]
    pub area: AreaRequest,
}

/// 地区列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaListRequest {
    /// 是否遍历子地区
    pub is_traverse: Option<bool>,
    /// 父级 ID
    pub parent_id: Option<i64>,
}
