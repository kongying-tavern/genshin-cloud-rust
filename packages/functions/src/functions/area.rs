use anyhow::Result;
use sea_orm::EntityTrait;

use crate::SharedDatabaseConnection;
use _database::models::area as Area;
use _utils::schemas::area::Schema as AreaSchema;

pub async fn list_area(db: &SharedDatabaseConnection) -> Result<Vec<AreaSchema>> {
    let mut res = Vec::<AreaSchema>::new();

    for cc in Area::Entity::find().all(&*db.conn).await? {
        if cc.del_flag != 0 {
            continue;
        }
        db.cache.area.insert(cc.id, cc.clone()).await;

        todo!("直接使用 AreaSchema 到 Area 自己的转换，.into()");
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

pub async fn get_area(db: &SharedDatabaseConnection, id: i64) -> Result<AreaSchema> {
    if db.cache.area.contains_key(&id) {
        let res = db
            .cache
            .area
            .get(&id)
            .ok_or(anyhow::anyhow!("Cache item has been deleted"))?;

        todo!("直接使用 AreaSchema 到 Area 自己的转换，.into()");
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
        .one(&*db.conn)
        .await?
        .ok_or(anyhow::anyhow!("Area not found"))?;
    if res.del_flag != 0 {
        return Err(anyhow::anyhow!("Area not found"));
    }

    db.cache.area.insert(res.id, res.clone()).await;
    todo!("直接使用 AreaSchema 到 Area 自己的转换，.into()");
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
