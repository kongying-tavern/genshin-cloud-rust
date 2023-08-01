use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "sys_role")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(version)]
    pub version: i64,
    #[sea_orm(create_time)]
    pub create_time: DateTime,
    #[sea_orm(update_time)]
    pub update_time: Option<DateTime>,

    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    #[sea_orm(default_value = 0)]
    pub del_flag: i16,

    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub authorized_grant_types: String,
    pub web_server_redirect_uri: String,
    pub authorities: String,

    pub access_token_validity: i64,
    pub refresh_token_validity: i64,
    pub additional_information: String,
    pub auto_approve: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
