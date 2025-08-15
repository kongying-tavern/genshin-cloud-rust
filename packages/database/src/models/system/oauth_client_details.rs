use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "sys_role", schema_name = "genshin_map")]
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

    /// 客户端 ID
    pub client_id: String,
    /// 客户端密钥
    pub client_secret: String,
    /// 权限范围
    pub scope: Option<String>,
    /// 鉴权类型
    pub authorized_grant_types: Option<String>,
    /// 重定向地址
    pub web_server_redirect_uri: Option<String>,
    /// 权限
    pub authorities: Option<String>,

    /// 授权密钥过期时间
    pub access_token_validity: Option<i32>,
    /// 刷新密钥过期时间
    pub refresh_token_validity: Option<i32>,
    /// 额外信息
    pub additional_information: Option<String>,
    /// 自动同意
    pub auto_approve: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
