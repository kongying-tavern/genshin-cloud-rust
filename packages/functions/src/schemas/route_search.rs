use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 路线名称的模糊搜索字段
    pub namePart: Option<String>,
    // 创建人昵称的模糊搜索字段，不能与创建人 ID 字段共存
    pub creatorNicknamePart: Option<String>,
    // 创建人 ID，不能与创建人昵称的模糊搜索字段共存
    pub creatorId: Option<String>,
    // 当前页，从 0 开始
    pub current: Option<i64>,
    // 每页大小，默认为 10
    pub size: Option<i64>,
}
