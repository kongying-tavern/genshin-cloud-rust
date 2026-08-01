use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{ActiveValue::Set, QueryFilter, QuerySelect, prelude::*};

use _database::{DB_CONN, models::common::route as route_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        route::{
            RouteAddRequest, RoutePageResponse, RouteSearchRequest, RouteUpdateRequest, RouteVO,
        },
        wrapper::{CommonResponse, Pagination},
    },
};

fn to_vo(r: route_model::Model) -> RouteVO {
    RouteVO {
        id: r.id,
        name: r.name,
        content: r.content,
        marker_list: r.marker_list.0,
        hidden_flag: r.hidden_flag,
        video: r.video,
        extra: r.extra,
        creator_nickname: r.creator_nickname,
        creator_id: r.creator_id,
        create_time: r.create_time,
    }
}

/// 新增路线
pub async fn do_add(_auth: AuthInfo, payload: RouteAddRequest) -> Result<i64> {
    let db = &DB_CONN.wait().pg_conn;
    let now = Utc::now().naive_utc();

    let marker_list: Vec<i64> = payload
        .marker_list
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();

    let am = route_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        name: Set(payload.name),
        content: Set(payload.content),
        marker_list: Set(route_model::MarkerListWrapper(marker_list)),
        hidden_flag: Set(payload.hidden_flag),
        video: Set(payload.video),
        extra: Set(serde_json::Value::Object(
            payload
                .extra
                .map(|m| {
                    m.into_iter()
                        .filter_map(|(k, v)| v.map(|vv| (k, vv)))
                        .collect()
                })
                .unwrap_or_default(),
        )),
        creator_nickname: Set(payload.creator_nickname.unwrap_or_default()),
    };

    let res = route_model::Entity::insert(am).exec(db).await?;
    Ok(res.last_insert_id)
}

/// 更新路线
pub async fn do_update(
    _auth: AuthInfo,
    payload: RouteUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let r = route_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let r = r.ok_or(anyhow!("Route not found"))?;
    let mut am: route_model::ActiveModel = r.into();

    let marker_list: Vec<i64> = payload
        .marker_list
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();

    am.name = Set(payload.name);
    am.content = Set(payload.content);
    am.marker_list = Set(route_model::MarkerListWrapper(marker_list));
    am.hidden_flag = Set(payload.hidden_flag);
    am.video = Set(payload.video);
    am.extra = Set(serde_json::Value::Object(
        payload
            .extra
            .map(|m| {
                m.into_iter()
                    .filter_map(|(k, v)| v.map(|vv| (k, vv)))
                    .collect()
            })
            .unwrap_or_default(),
    ));
    am.creator_nickname = Set(payload.creator_nickname.unwrap_or_default());

    route_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 分页查询路线
pub async fn do_get_page(
    _auth: AuthInfo,
    payload: Pagination,
) -> Result<CommonResponse<RoutePageResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = route_model::Entity::find_safety().count(db).await?;
    let items = route_model::Entity::find_safety()
        .limit(size)
        .offset(offset)
        .all(db)
        .await?
        .into_iter()
        .map(to_vo)
        .collect();
    Ok(CommonResponse::new(Ok(RoutePageResponse {
        total: total as i64,
        items,
    })))
}

/// 按创建人/名称搜索路线
pub async fn do_get_search(
    _auth: AuthInfo,
    payload: RouteSearchRequest,
) -> Result<RoutePageResponse> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = route_model::Entity::find_safety();

    if let Ok(cid) = payload.creator_id.unwrap_or_default().parse::<i64>() {
        query = query.filter(route_model::Column::CreatorId.eq(cid));
    }
    if let Some(nickname) = payload.creator_nickname_part {
        query = query.filter(route_model::Column::CreatorNickname.like(format!("%{}%", nickname)));
    }
    if let Some(name) = payload.name_part {
        query = query.filter(route_model::Column::Name.like(format!("%{}%", name)));
    }

    let size = payload.page.size.unwrap_or(10) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await?;
    let items = query
        .limit(size)
        .offset(offset)
        .all(db)
        .await?
        .into_iter()
        .map(to_vo)
        .collect();
    Ok(RoutePageResponse {
        total: total as i64,
        items,
    })
}

/// 按 ID 列表批量查询路线
pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    payload: Vec<f64>,
) -> Result<CommonResponse<Vec<RouteVO>>> {
    let db = &DB_CONN.wait().pg_conn;
    let ids: Vec<i64> = payload.iter().map(|f| *f as i64).collect();

    let items = route_model::Entity::find_safety()
        .filter(route_model::Column::Id.is_in(ids))
        .all(db)
        .await?
        .into_iter()
        .map(to_vo)
        .collect();
    Ok(CommonResponse::new(Ok(items)))
}

/// 软删除路线
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let r = route_model::Entity::find_safety_by_id(id).one(db).await?;
    let r = r.ok_or(anyhow!("Route not found"))?;
    let mut am: route_model::ActiveModel = r.into();
    am.del_flag = Set(true);
    route_model::Entity::delete_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
