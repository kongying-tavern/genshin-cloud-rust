use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 乐观锁，修改次数
    pub version: Option<i64>,
    // 分类 ID
    pub id: Option<i64>,
    // 分类名
    pub name: Option<String>,
    // 父级分类 ID，-1 为根分类
    pub parent: Option<i64>,
    // 是否为末端类型
    pub isFinal: Option<bool>,
}
