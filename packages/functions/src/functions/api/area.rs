use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{AreaAddRequest, AreaListRequest, AreaUpdateRequest},
};

// 新增地区
pub async fn do_add(_auth: AuthInfo, _payload: AreaAddRequest) -> Result<i64> {
    // TODO: 使用 database::DB_CONN 插入并返回新 id
    Ok(0)
}

// 更新地区
pub async fn do_update(_auth: AuthInfo, _payload: AreaUpdateRequest) -> Result<()> {
    // TODO: 更新逻辑
    Ok(())
}

// 列表
pub async fn do_list(_auth: AuthInfo, _payload: AreaListRequest) -> Result<Vec<serde_json::Value>> {
    // TODO: 查询逻辑，返回列表
    Ok(vec![])
}

// 获取单个
pub async fn do_get(_auth: AuthInfo, _area_id: i64) -> Result<serde_json::Value> {
    // TODO: 查询单个
    Ok(serde_json::json!({}))
}

// 删除
pub async fn do_delete(_auth: AuthInfo, _area_id: i64) -> Result<()> {
    // TODO: 删除逻辑
    Ok(())
}
