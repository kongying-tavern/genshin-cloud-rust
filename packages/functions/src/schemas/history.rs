use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 编号
    pub id: i64,
    // 内容
    pub content: String,
    // MD5
    pub md5: String,
    // 类型 ID
    pub tId: i64,
    // 记录类型
    pub r#type: i32,
    // IPV4
    pub ipv4: String,
    // 创建人
    pub creatorId: i64,
    // 创建时间
    pub createTime: NaiveDateTime,
}
