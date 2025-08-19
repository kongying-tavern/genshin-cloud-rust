use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OauthLoginResponse {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 令牌类型
    pub token_type: OauthTokenType,
    /// 令牌寿命
    pub expires_in: u64,
    /// 生效范围
    pub scope: OauthScopeType,
    /// 唯一标识
    pub jti: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OauthAnonymousResponse {
    /// 访问令牌
    pub access_token: String,
    /// 令牌类型
    pub token_type: OauthTokenType,
    /// 令牌寿命
    pub expires_in: u64,
    /// 生效范围
    pub scope: OauthScopeType,
    /// 唯一标识
    pub jti: Uuid,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, EnumIter, Display, AsRefStr, Serialize, Deserialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OauthTokenType {
    /// Bearer
    #[default]
    Bearer,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, EnumIter, Display, AsRefStr, Serialize, Deserialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OauthScopeType {
    /// 全局
    #[default]
    All,
}
