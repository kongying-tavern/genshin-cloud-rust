use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 设备状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum ActionLogAction {
    #[serde(rename = "LOGIN")]
    Login,
}
