use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 乐观锁，修改次数
    pub version: Option<i64>,
    // 路线 ID
    pub id: Option<i64>,
    // 路线名称
    pub name: Option<String>,
    // 路线描述
    pub content: Option<String>,
    // 点位顺序数组
    pub marketList: Option<String>,
    // 显隐等级
    pub hiddenFlag: Option<i32>,
    // 视频地址
    pub video: Option<String>,
    // 额外信息
    pub extra: Option<String>,
    // 创建人
    pub creatorId: Option<i64>,
    // 创建人昵称
    pub creatorNickname: Option<String>,
}
