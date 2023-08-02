use anyhow::Result;
use lazy_static::lazy_static;
use std::cell::RefCell;
use tokio::sync::Mutex;

use moka::future::Cache;
use sea_orm::entity::EntityTrait;

use _database::{models::area::area as Area, DB_CONN};
use _utils::schemas::area::Schema as AreaSchema;

lazy_static! {
    static ref CACHE_AREA: Mutex<RefCell<Cache<i64, Area::Model>>> =
        Mutex::new(RefCell::new(Cache::new(1_000)));
}

pub async fn list_area() -> Result<Vec<AreaSchema>> {
    let db = DB_CONN.lock().await.get_mut().clone();

    let mut res = Vec::<AreaSchema>::new();

    for cc in Area::Entity::find().all(&db).await? {
        if cc.del_flag != 0 {
            continue;
        }
        CACHE_AREA
            .lock()
            .await
            .get_mut()
            .insert(cc.id, cc.clone())
            .await;

        // TODO - 直接使用 AreaSchema 到 Area 自己的转换
        res.push(AreaSchema {
            name: Some(cc.name),
            areaId: Some(cc.id),
            code: cc.code,
            content: cc.content,
            iconTag: Some(cc.icon_tag),
            parentId: Some(cc.parent_id),
            isFinal: Some(cc.is_final),
            hiddenFlag: Some(cc.hidden_flag),
            sortIndex: Some(cc.sort_index),
            specialFlag: Some(0),
        });
    }

    Ok(res)
}

pub async fn get_area(id: i64) -> Result<AreaSchema> {
    let db = DB_CONN.lock().await.get_mut().clone();

    if CACHE_AREA.lock().await.get_mut().contains_key(&id) {
        let res = CACHE_AREA
            .lock()
            .await
            .get_mut()
            .get(&id)
            .ok_or(anyhow::anyhow!("Cache item has been deleted"))?;

        // TODO - 直接使用 AreaSchema 到 Area 自己的转换
        return Ok(AreaSchema {
            name: Some(res.name),
            areaId: Some(res.id),
            code: res.code,
            content: res.content,
            iconTag: Some(res.icon_tag),
            parentId: Some(res.parent_id),
            isFinal: Some(res.is_final),
            hiddenFlag: Some(res.hidden_flag),
            sortIndex: Some(res.sort_index),
            specialFlag: Some(0),
        });
    }

    let res = Area::Entity::find_by_id(id)
        .one(&db)
        .await?
        .ok_or(anyhow::anyhow!("Area not found"))?;
    if res.del_flag != 0 {
        return Err(anyhow::anyhow!("Area not found"));
    }

    CACHE_AREA
        .lock()
        .await
        .get_mut()
        .insert(res.id, res.clone())
        .await;
    // TODO - 直接使用 AreaSchema 到 Area 自己的转换
    Ok(AreaSchema {
        name: Some(res.name),
        areaId: Some(res.id),
        code: res.code,
        content: res.content,
        iconTag: Some(res.icon_tag),
        parentId: Some(res.parent_id),
        isFinal: Some(res.is_final),
        hiddenFlag: Some(res.hidden_flag),
        sortIndex: Some(res.sort_index),
        specialFlag: Some(0),
    })
}
