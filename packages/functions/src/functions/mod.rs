pub mod api;
pub mod system;

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonResponse<T> {
    pub error: bool,
    pub error_status: u16,
    pub error_data: Option<serde_json::Value>,
    pub message: String,
    pub data: Option<T>,
    pub time: NaiveDateTime,
}

impl<T> CommonResponse<T> {
    pub fn new(result: Result<T>) -> Self {
        match result {
            Ok(data) => Self {
                error: false,
                data: Some(data),
                ..Default::default()
            },
            Err(err) => Self {
                error: true,
                message: err.to_string(),
                ..Default::default()
            },
        }
    }
}

impl<T> Default for CommonResponse<T> {
    fn default() -> Self {
        Self {
            error: false,
            error_status: 200,
            error_data: None,
            message: "".to_string(),
            data: None,
            time: chrono::Local::now().naive_local(),
        }
    }
}
