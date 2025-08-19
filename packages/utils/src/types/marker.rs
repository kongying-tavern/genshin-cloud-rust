use serde::{Deserialize, Serialize};
use strum::EnumIter;

use sea_orm::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum MarkerLinkageLinkAction {
    /// 触发
    #[sea_orm(string_value = "TRIGGER")]
    Trigger,
    /// 全部触发
    #[sea_orm(string_value = "TRIGGER_ALL")]
    TriggerAll,
    /// 任意触发
    #[sea_orm(string_value = "TRIGGER_ANY")]
    TriggerAny,
    /// 相关
    #[sea_orm(string_value = "RELATED")]
    Related,
    /// 有向
    #[sea_orm(string_value = "DIRECTED")]
    Directed,
    /// 单向路径
    #[sea_orm(string_value = "PATH_UNI_DIR")]
    PathUniDir,
    /// 双向路径
    #[sea_orm(string_value = "PATH_BI_DIR")]
    PathBiDir,
    /// 等价
    #[sea_orm(string_value = "EQUIVALENT")]
    Equivalent,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MarkerPunctuateStatus {
    /// 暂存
    #[sea_orm(num_value = 0)]
    Pending = 0,
    /// 审核中
    #[sea_orm(num_value = 1)]
    Reviewing = 1,
    /// 不通过
    #[sea_orm(num_value = 2)]
    Rejected = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MarkerPunctuateMethodType {
    /// 新增
    #[sea_orm(num_value = 1)]
    Added = 1,
    /// 修改
    #[sea_orm(num_value = 2)]
    Modified = 2,
    /// 删除
    #[sea_orm(num_value = 3)]
    Deleted = 3,
}
