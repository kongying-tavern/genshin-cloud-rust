use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "oauth_client_details", schema_name = "genshin_map")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub version: i64,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub create_time: DateTime,
    pub update_time: Option<DateTime>,
    pub del_flag: i16,

    pub client_id: String,
    pub client_secret: String,
    pub scope: Option<String>,
    pub authorized_grant_types: Option<String>,
    pub web_server_redirect_uri: Option<String>,
    pub authorities: Option<String>,
    pub access_token_validity: Option<i32>,
    pub refresh_token_validity: Option<i32>,
    pub additional_information: Option<String>,
    pub auto_approve: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
