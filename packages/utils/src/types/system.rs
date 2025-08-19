use serde::{Deserialize, Serialize};
use strum::EnumIter;

use sea_orm::prelude::*;

use super::HiddenFlag;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum SystemActionLogAction {
    /// 登录
    #[sea_orm(string_value = "LOGIN")]
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum SystemUserRole {
    /// 系统管理员
    Admin = 0,
    /// 地图管理员
    MapManager = 1,
    /// 测试打点员
    MapNeigui = 2,
    /// 地图打点员
    MapPunctuate = 3,
    /// 地图用户
    MapUser = 4,
    /// 匿名用户
    Visitor = 5,
}

impl SystemUserRole {
    fn is_available(self, flag: HiddenFlag) -> bool {
        if matches!(flag, HiddenFlag::Visible | HiddenFlag::Suprise) {
            return true;
        }

        match self {
            SystemUserRole::Admin | SystemUserRole::MapNeigui => {
                matches!(flag, HiddenFlag::Hidden | HiddenFlag::Spy)
            }
            SystemUserRole::MapManager | SystemUserRole::MapPunctuate => {
                matches!(flag, HiddenFlag::Hidden)
            }
            SystemUserRole::MapUser | SystemUserRole::Visitor => false,
        }
    }
}
