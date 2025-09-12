use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{TagAddRequest, TagListRequest, TagUpdateTypeRequest},
};
use serde_json::Value as JsonValue;

pub async fn do_create(_auth: AuthInfo, _tag_name: String) -> Result<i64> {
    // TODO: 创建空标签并返回 id
    Ok(0)
}

pub async fn do_list(_auth: AuthInfo, _payload: TagListRequest) -> Result<serde_json::Value> {
    // TODO: 返回标签列表
    Ok(serde_json::json!({}))
}

pub async fn do_get_single(_auth: AuthInfo, _tag_id: i64) -> Result<serde_json::Value> {
    // TODO: 获取单个标签
    Ok(serde_json::json!({}))
}

pub async fn do_delete(_auth: AuthInfo, _tag_id: i64) -> Result<()> {
    // TODO: 删除标签
    Ok(())
}

pub async fn do_update_type(_auth: AuthInfo, _payload: TagUpdateTypeRequest) -> Result<()> {
    // TODO: 更新标签类型
    Ok(())
}

pub async fn do_update_association(_auth: AuthInfo, _payload: JsonValue) -> Result<()> {
    // TODO: 更新关联（utils 中没有 TagUpdateAssociationRequest，这里先接受通用的 JSON）
    Ok(())
}
