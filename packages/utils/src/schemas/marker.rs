use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 点位 ID
    pub id: Option<i64>,
    // 点位名称
    pub markerTitle: Option<String>,
    // 点位坐标
    pub position: Option<String>,
    // 点位物品列表
    pub itemList: Option<Vec<super::marker_item_link::Schema>>,
    // 点位说明
    pub content: Option<String>,
    // 点位图片
    pub picture: Option<String>,
    // 点位初始标记者
    pub markerCreatorId: Option<i64>,
    // 点位图片上传者
    pub pictureCreatorId: Option<i64>,
    // 点位视频
    pub videoPath: Option<String>,
    // 额外特殊字段
    pub extra: Option<String>,
    // 刷新时间
    pub refreshTime: Option<i64>,
    // 隐藏标志
    pub hiddenFlag: Option<i32>,
}
