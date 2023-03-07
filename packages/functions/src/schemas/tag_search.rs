use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 标签名列表
    pub tagList: Option<Vec<String>>,
    // 图标标签分类列表
    pub typeIdList: Option<Vec<i64>>,
    // 当前页，从 0 开始
    pub current: Option<i64>,
    // 每页大小，默认为 10
    pub size: Option<i64>,
}

impl DTO {
    #[allow(dead_code)]
    fn default() -> Self {
        Self {
            tagList: None,
            typeIdList: None,
            current: Some(0),
            size: Some(10),
        }
    }
}
