use serde::{Deserialize, Serialize};

/// 设备状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionLogAction {
    #[serde(rename = "LOGIN")]
    Login,
}

// ActionLogSort 和 ActionLogParams 已移动到对应的路由文件中
