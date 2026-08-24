use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OauthLoginResponse {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 令牌类型
    pub token_type: OauthTokenType,
    /// 有效期（秒）
    pub expires_in: i64,
    /// 有效范围
    pub scope: OauthScopeType,
    /// 唯一标识
    pub jti: Uuid,
    /// 用户 ID（Java 契约：OAuth 额外信息，camelCase）
    #[serde(rename = "userId")]
    pub user_id: i64,
    /// 角色 code 列表（Java 契约，camelCase）
    #[serde(rename = "userRoles")]
    pub user_roles: Vec<String>,
    /// 环境标识（Java 契约，camelCase）
    #[serde(rename = "env")]
    pub env: Option<String>,
    /// 登录提示消息（如设备/IP 波动警告，Java 契约，camelCase）
    #[serde(rename = "message")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OauthAnonymousResponse {
    /// 访问令牌
    pub access_token: String,
    /// 令牌类型
    pub token_type: OauthTokenType,
    /// 令牌寿命
    pub expires_in: i64,
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
