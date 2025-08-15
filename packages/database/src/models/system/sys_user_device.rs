use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "sys_user_device", schema_name = "genshin_map")]
pub struct Model {
    /// 乐观锁
    pub version: i64,
    /// ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 创建时间
    pub create_time: DateTime,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 创建人
    pub creator_id: Option<i64>,
    /// 更新人
    pub updater_id: Option<i64>,
    /// 逻辑删除
    pub del_flag: bool,

    /// 用户 ID
    pub user_id: Option<i64>,
    /// 设备编码
    /// 记录设备 User Agent 信息
    pub device_id: String,
    /// IPv4
    pub ipv4: Option<String>,
    /// 设备状态
    pub status: i32,
    /// 上次登录时间
    pub last_login_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
