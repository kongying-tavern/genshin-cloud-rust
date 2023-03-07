use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 当前页，从 0 开始
    pub current: Option<i64>,
    // 每页大小，默认为10
    pub size: Option<i64>,
    // 记录类型，不传时默认查询全部类型
    pub r#type: Option<i32>,
    // 记录类型，不传时默认查询全部数据
    pub id: Option<Vec<i32>>,
}

impl DTO {
    #[allow(dead_code)]
    fn default() -> Self {
        Self {
            current: Some(0),
            size: Some(10),
            r#type: None,
            id: Some(vec![]),
        }
    }
}
