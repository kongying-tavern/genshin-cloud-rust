use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PunctuateStatus {
    STAGE = 0,  // 暂存
    COMMIT = 1, // 审核中
    REJECT = 2, // 驳回
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PunctuateMethod {
    ADD = 1,    // 新增
    UPDATE = 2, // 更新
    DELETE = 3, //删除
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 打点 ID
    pub punctuateId: Option<i64>,
    // 原有点位 ID
    pub originalMarkerId: Option<i64>,
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
    pub markerExtraContent: Option<String>,
    // 点位提交者 ID
    pub author: Option<i64>,
    // 审核状态
    pub status: Option<PunctuateStatus>,
    // 审核备注
    pub auditRemark: Option<String>, // 数据库列名为 audit_remark
    // 操作类型
    pub methodType: Option<PunctuateMethod>,
    // 刷新时间
    pub refreshTime: Option<i64>,
    // 隐藏标志
    pub hiddenFlag: Option<i32>,
}
