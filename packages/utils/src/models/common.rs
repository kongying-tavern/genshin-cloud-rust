use serde::{Deserialize, Serialize};

/// 通用空响应（用于占位的无数据响应）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptyResponse {}
