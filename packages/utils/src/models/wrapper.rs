use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::SysUserVO;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonResponse<T> {
    pub error: bool,
    pub error_status: u16,
    pub error_data: Option<serde_json::Value>,
    pub message: String,
    pub data: Option<T>,
    pub users: Vec<SysUserVO>,
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

    pub fn with_users(mut self, users: Vec<SysUserVO>) -> Self {
        self.users = users;
        self
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.error_status = status;
        self
    }

    pub fn with_error_data(mut self, error_data: serde_json::Value) -> Self {
        self.error_data = Some(error_data);
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
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
            users: Vec::new(),
            time: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub current: u32,
    pub size: u32,
}
