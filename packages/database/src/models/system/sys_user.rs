use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::{
    impl_safe_operation,
    models::SysUserVO,
    types::{AccessPolicyList, SystemUserRole},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "sys_user", schema_name = "genshin_map")]
pub struct Model {
    /// 乐观锁
    pub version: u64,
    /// ID
    #[sea_orm(primary_key)]
    pub id: u64,
    /// 创建时间
    pub create_time: DateTime,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 创建人
    pub creator_id: Option<u64>,
    /// 更新人
    pub updater_id: Option<u64>,
    /// 逻辑删除
    pub del_flag: bool,

    /// 用户名
    #[sea_orm(indexed)]
    pub username: String,
    /// 密码
    /// 格式形如 {bcrypt}$2a$10$xxxxxxxxxxxxxxxxxxxx
    pub password: String,
    /// 昵称
    pub nickname: Option<String>,
    /// QQ
    #[sea_orm(indexed)]
    pub qq: Option<String>,
    /// 手机号
    #[sea_orm(indexed)]
    pub phone: Option<String>,
    /// 头像链接
    pub logo: Option<String>,

    /// 角色
    pub role_id: SystemUserRole,
    /// 权限策略
    #[sea_orm(column_type = "Json")]
    pub access_policy: AccessPolicyList,
    /// 备注
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::system::sys_user::Entity",
        from = "Column::CreatorId",
        to = "super::super::system::sys_user::Column::Id"
    )]
    CreatorId,
    #[sea_orm(
        belongs_to = "super::super::system::sys_user::Entity",
        from = "Column::UpdaterId",
        to = "super::super::system::sys_user::Column::Id"
    )]
    UpdaterId,
}

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}

impl Into<SysUserVO> for Model {
    fn into(self) -> SysUserVO {
        SysUserVO {
            id: self.id,
            username: self.username,
            nickname: self.nickname,
            qq: self.qq,
            phone: self.phone,
            logo: self.logo,
            role_id: self.role_id,
            access_policy: self.access_policy,
            remark: self.remark,
        }
    }
}
