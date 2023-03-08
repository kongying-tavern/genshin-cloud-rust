pub mod area;

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct RequestData<T> {
    pub error: Option<bool>,
    pub errorStatus: Option<i32>,
    pub errorData: Option<()>, // 这儿真没法写，咱不像 JVAV 那样有泛型擦除
    pub message: Option<String>,
    pub data: Option<T>,
    pub time: Option<NaiveDateTime>,
}

impl<T> RequestData<T> {
    pub fn new(result: Result<T>) -> Self {
        match result {
            Ok(data) => Self {
                error: Some(false),
                data: Some(data),
                ..Default::default()
            },
            Err(err) => Self {
                error: Some(true),
                message: Some(err.to_string()),
                ..Default::default()
            },
        }
    }
}

impl<T> Default for RequestData<T> {
    fn default() -> Self {
        Self {
            error: None,
            errorStatus: None,
            errorData: None,
            message: None,
            data: None,
            time: Some(chrono::Local::now().naive_local()),
        }
    }
}

pub const DEFAULT_ERROR_JSON_MSG: &str = r#"{ "error": true, "message": "Unknown error" }"#;
