use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder, QuerySelect, prelude::*};

use _database::{
    DatabaseConnectionMap,
    models::common::{history as history_model, score_stat as score_stat_model},
    models::item::item as item_model,
    models::marker::{marker as marker_model, marker_item_link as mil_model},
    models::system::sys_user as sys_user_model,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::score::{ScoreDataRequest, ScoreGenerateRequest},
    models::wrapper::CommonResponse,
    types::HistoryOperationType,
};

/// 生成评分统计数据（批处理管线，Java `ScoreGenerateService` 语义）。
///
/// 1. 清除 score_stat 中该 scope+span+时间范围的旧数据；
/// 2. 取区间内的点位历史（type=4），点位全集 = 历史涉及 ∪ 区间内新建；
/// 3. 每个点位补齐首尾比对条目（新建点位以首个关联物品合成初始值；
///    区间后首条历史或点位现状合成末值）；
/// 4. 时间线内**相邻对**差分：字段差异计 `fields[field] += 1`（忽略
///    version/id/markerCreatorId/pictureCreatorId/itemList/log），
///    `chars.markerTitle/content += Levenshtein(前值, 后值)`；
/// 5. 记账归属「前一条记录的创建者 × 该时刻的 Asia/Shanghai 日桶」；
/// 6. 每个 (贡献者, 日桶) 写一行 score_stat，content = {fields, chars}
///    （Java `ScoreDataPunctuateVo` 形态，读路径直接合并）。
pub async fn do_generate_score(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    payload: ScoreGenerateRequest,
) -> Result<CommonResponse<String>> {
    auth.require_non_anonymous()?;
    let db = &db.pg_conn;

    // 解析时间范围（Java calculateSpan/DAY：对齐到 Asia/Shanghai 日界）。
    let span_start = timestamp_to_naive(payload.start_time);
    let span_end = timestamp_to_naive(payload.end_time);
    let now = Utc::now().naive_utc();
    let scope = &payload.scope;
    let span_name = &payload.span;

    // 1. 批量软删除旧数据（该 scope + span + 时间范围内的 score_stat 行）
    let old_ids: Vec<i64> = score_stat_model::Entity::find_safety()
        .filter(score_stat_model::Column::Scope.eq(scope))
        .filter(score_stat_model::Column::Span.eq(span_name))
        .filter(score_stat_model::Column::SpanStartTime.gte(span_start))
        .filter(score_stat_model::Column::SpanEndTime.lte(span_end))
        .all(db)
        .await?
        .into_iter()
        .map(|m| m.id)
        .collect();

    if !old_ids.is_empty() {
        score_stat_model::Entity::update_many()
            .col_expr(
                score_stat_model::Column::DelFlag,
                sea_orm::sea_query::Expr::value(true),
            )
            .filter(score_stat_model::Column::Id.is_in(old_ids))
            .exec(db)
            .await?;
    }

    // 2. 区间内的点位历史（type=4；Java getHistoryList）
    let histories = history_model::Entity::find_safety()
        .filter(history_model::Column::HistoryType.eq(HistoryOperationType::Position))
        .filter(history_model::Column::CreateTime.gte(span_start))
        .filter(history_model::Column::CreateTime.lte(span_end))
        .order_by_asc(history_model::Column::CreateTime)
        .all(db)
        .await?;

    // 3. 点位全集 = 历史涉及的点位 ∪ 区间内新建的点位（Java markerFullList）
    let mut marker_ids: Vec<i64> = histories.iter().map(|h| h.t_id).collect();
    let created_markers = marker_model::Entity::find_safety()
        .filter(marker_model::Column::CreateTime.gte(span_start))
        .filter(marker_model::Column::CreateTime.lte(span_end))
        .all(db)
        .await?;
    marker_ids.extend(created_markers.iter().map(|m| m.id));
    marker_ids.sort_unstable();
    marker_ids.dedup();
    if marker_ids.is_empty() {
        return Ok(CommonResponse::new(Ok("ok".to_string())));
    }
    let mut markers: std::collections::HashMap<i64, marker_model::Model> =
        std::collections::HashMap::new();
    for chunk in marker_ids.chunks(1000) {
        for m in marker_model::Entity::find_safety()
            .filter(marker_model::Column::Id.is_in(chunk.to_vec()))
            .all(db)
            .await?
        {
            markers.insert(m.id, m);
        }
    }

    // 4. 初始化比对条目（Java getInitializeHistory）：区间内**新建**的点位，
    //    以其首个关联物品的名称/默认说明合成「变更前」记录。
    // 每个点位的**首个**关联物品（Java：mil 按 create_time、id 升序取第一条）。
    let mils = mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.is_in(marker_ids.clone()))
        .order_by_asc(mil_model::Column::CreateTime)
        .order_by_asc(mil_model::Column::Id)
        .all(db)
        .await?;
    let mut first_item: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    for l in mils {
        first_item.entry(l.marker_id).or_insert(l.item_id);
    }
    let mut items: std::collections::HashMap<i64, item_model::Model> =
        std::collections::HashMap::new();
    {
        let ids: Vec<i64> = first_item.values().copied().collect();
        for chunk in ids.chunks(1000) {
            for it in item_model::Entity::find_safety()
                .filter(item_model::Column::Id.is_in(chunk.to_vec()))
                .all(db)
                .await?
            {
                items.insert(it.id, it);
            }
        }
    }

    // 5. 末次比对条目（Java getFinalizeHistory）：区间后首条历史优先；
    //    无则由点位现状合成（creator=updater，time=update+100s）。
    let post_histories = history_model::Entity::find_safety()
        .filter(history_model::Column::HistoryType.eq(HistoryOperationType::Position))
        .filter(history_model::Column::CreateTime.gt(span_end))
        .filter(history_model::Column::TId.is_in(marker_ids.clone()))
        .order_by_asc(history_model::Column::CreateTime)
        .all(db)
        .await?;
    let mut post_first: std::collections::HashMap<i64, &history_model::Model> =
        std::collections::HashMap::new();
    for h in &post_histories {
        post_first.entry(h.t_id).or_insert(h);
    }

    // ── 组装每个点位的时间线并做相邻对差分（Java getHistoryFieldDiff）──
    struct Rec {
        creator_id: Option<i64>,
        create_time: chrono::NaiveDateTime,
        content: serde_json::Value,
    }
    let parse = |raw: &str| {
        serde_json::from_str::<serde_json::Value>(raw)
            .ok()
            .unwrap_or(serde_json::Value::Null)
    };
    let mut timelines: std::collections::HashMap<i64, Vec<Rec>> = std::collections::HashMap::new();
    for h in &histories {
        timelines.entry(h.t_id).or_default().push(Rec {
            creator_id: h.creator_id,
            create_time: h.create_time,
            content: parse(&h.content),
        });
    }
    for m in created_markers.iter() {
        let item = first_item.get(&m.id).and_then(|iid| items.get(iid));
        timelines.entry(m.id).or_default().push(Rec {
            creator_id: m.creator_id,
            create_time: m.create_time,
            content: serde_json::json!({
                "id": m.id,
                "markerTitle": item.map(|it| it.name.clone()).unwrap_or_default(),
                "content": item
                    .and_then(|it| it.default_content.clone())
                    .unwrap_or_default(),
            }),
        });
    }
    for (mid, marker) in &markers {
        let finalize_rec = match post_first.get(mid) {
            Some(h) => Rec {
                creator_id: h.creator_id,
                create_time: h.create_time,
                content: parse(&h.content),
            },
            None => Rec {
                creator_id: marker.updater_id,
                create_time: (marker.update_time.unwrap_or(marker.create_time).and_utc()
                    + chrono::Duration::seconds(100))
                .naive_utc(),
                content: serde_json::json!({
                    "version": marker.version,
                    "id": marker.id,
                    "markerStamp": marker.marker_stamp,
                    "markerTitle": marker.marker_title,
                    "position": marker.position,
                    "content": marker.content,
                    "picture": marker.picture,
                    "videoPath": marker.video_path,
                    "refreshTime": marker.refresh_time,
                    "hiddenFlag": marker.hidden_flag as i32,
                    "extra": marker.extra,
                }),
            },
        };
        timelines.entry(*mid).or_default().push(finalize_rec);
    }

    // 差分记账：(creator, 日桶) → {fields: {字段: 次数}, chars: {字段: 字数}}
    type Bucket = (
        serde_json::Map<String, serde_json::Value>,
        serde_json::Map<String, serde_json::Value>,
    );
    let mut score: std::collections::BTreeMap<(i64, chrono::NaiveDateTime), Bucket> =
        std::collections::BTreeMap::new();
    for recs in timelines.values_mut() {
        recs.sort_by_key(|r| r.create_time);
        for pair in recs.windows(2) {
            let (before, after) = (&pair[0], &pair[1]);
            let before_obj = before.content.as_object().cloned().unwrap_or_default();
            let after_obj = after.content.as_object().cloned().unwrap_or_default();
            let key = (
                before.creator_id.unwrap_or(0),
                day_bucket_utc(before.create_time),
            );
            let (fields, chars) = score
                .entry(key)
                .or_insert_with(|| (serde_json::Map::new(), serde_json::Map::new()));
            // 1) 字段差异（Java DiffUtils.getFieldsDiff + getDiffConfig）
            let mut keys: Vec<&String> = before_obj.keys().collect();
            for k in after_obj.keys() {
                if !before_obj.contains_key(k) {
                    keys.push(k);
                }
            }
            for k in keys {
                if SCORE_IGNORED_FIELDS.contains(&k.as_str()) {
                    continue;
                }
                let b = normalized_field(k, before_obj.get(k).unwrap_or(&serde_json::Value::Null));
                let a = normalized_field(k, after_obj.get(k).unwrap_or(&serde_json::Value::Null));
                if b != a {
                    let e = fields.entry(k.clone()).or_insert(0.into());
                    *e = serde_json::Value::from(e.as_i64().unwrap_or(0) + 1);
                }
            }
            // 2) 文本差异（Java DiffUtils.Levenshtein 距离）
            for field in ["markerTitle", "content"] {
                let b = before_obj.get(field).and_then(|v| v.as_str()).unwrap_or("");
                let a = after_obj.get(field).and_then(|v| v.as_str()).unwrap_or("");
                let d = levenshtein(b, a);
                if d > 0 {
                    let e = chars.entry(field.to_string()).or_insert(0.into());
                    *e = serde_json::Value::from(e.as_i64().unwrap_or(0) + d as i64);
                }
            }
        }
    }

    // 6. 写入 score_stat（每个 (贡献者, 日桶) 一行，content = {fields, chars}）
    for ((user_id, bucket_start), (fields, chars)) in score {
        let bucket_end = bucket_start + chrono::Duration::seconds(86_399);
        let am = score_stat_model::ActiveModel {
            version: Set(1),
            id: sea_orm::ActiveValue::NotSet,
            create_time: Set(now),
            update_time: Set(Some(now)),
            creator_id: Set(Some(user_id)),
            updater_id: Set(Some(user_id)),
            del_flag: Set(false),
            scope: Set(scope.clone()),
            span: Set(span_name.clone()),
            span_start_time: Set(bucket_start),
            span_end_time: Set(bucket_end),
            user_id: Set(Some(user_id)),
            content: Set(Some(serde_json::json!({
                "fields": fields,
                "chars": chars,
            }))),
        };
        score_stat_model::Entity::insert(am).exec(db).await?;
    }

    Ok(CommonResponse::new(Ok("ok".to_string())))
}

/// 字段差分的忽略清单（Java `getDiffConfig().setIgnore`）。
const SCORE_IGNORED_FIELDS: &[&str] = &[
    "log",
    "version",
    "id",
    "markerCreatorId",
    "pictureCreatorId",
    "itemList",
];

/// 字段值归一（Java `ActionsPre`）：hiddenFlag/refreshTime 按整数、
/// 文本字段按字符串比较（数字 3 与 "3" 视为相等）。
fn normalized_field(key: &str, v: &serde_json::Value) -> serde_json::Value {
    match key {
        "hiddenFlag" | "refreshTime" => {
            if let Some(n) = v.as_i64() {
                return serde_json::Value::from(n);
            }
            if let Some(n) = v.as_str().and_then(|s| s.parse::<i64>().ok()) {
                return serde_json::Value::from(n);
            }
            if let Some(f) = v.as_f64() {
                return serde_json::Value::from(f as i64);
            }
            v.clone()
        },
        "position" | "markerTitle" | "content" | "picture" | "videoPath" => v
            .as_str()
            .map(serde_json::Value::from)
            .unwrap_or_else(|| v.clone()),
        _ => v.clone(),
    }
}

/// Levenshtein 编辑距离（Java `DiffUtils.Levenshtein` 语义，两行 DP）。
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// 时间戳所属的 Asia/Shanghai 日桶（起点，UTC 表示）。
/// Java `calibrateSpan(DAY, ts)`：按 +08:00 日历日对齐，桶 = 该日
/// 00:00:00..23:59:59，存储为 UTC。
fn day_bucket_utc(t: chrono::NaiveDateTime) -> chrono::NaiveDateTime {
    use chrono::TimeZone;
    let local = chrono::FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .from_utc_datetime(&t);
    let day = local.date_naive();
    day.and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::FixedOffset::east_opt(8 * 3600).unwrap())
        .single()
        .map(|dt| dt.naive_utc())
        .unwrap_or(t)
}

/// 读取评分统计数据——从 score_stat 表查询真实聚合记录，并按用户聚合为
/// 前端 `ScoreVo[]`（`{userId, user, data{chars,fields}, scope, span}`）结构。
///
/// content JSON 兼容两种来源：
/// - Java 形态：`{fields: {字段: 次数}, chars: {字段: 字数}}`，直接合并；
/// - 本服务 `do_generate_score` 写入的简化形态 `{type, count, fieldWeight}`：
///   编辑次数归入 `fields.content`，加权得分（取整）归入 `chars.content`。
pub async fn do_get_score_data(
    db: &DatabaseConnectionMap,
    _auth: AuthInfo,
    payload: ScoreDataRequest,
) -> Result<CommonResponse<serde_json::Value>> {
    let db = &db.pg_conn;

    let start = timestamp_to_naive(payload.start_time);
    let end = timestamp_to_naive(payload.end_time);

    let stats = score_stat_model::Entity::find_safety()
        .filter(score_stat_model::Column::Scope.eq(&payload.scope))
        .filter(score_stat_model::Column::Span.eq(&payload.span))
        .filter(score_stat_model::Column::SpanStartTime.gte(start))
        .filter(score_stat_model::Column::SpanEndTime.lte(end))
        // 上限 10 万：原 10_000 会静默截断——单 span 贡献者 >1 万时排行缺人
    // （round3 L13）。保留上限仅为兜底异常超大响应，正常数据量远达不到。
    .limit(100_000)
        .all(db)
        .await?;

    // 按用户聚合（同一用户跨多个时间桶的行合并，对齐 Java `scoreDataMap.merge`）
    type ScoreAgg = (
        serde_json::Map<String, serde_json::Value>,
        serde_json::Map<String, serde_json::Value>,
    );
    let mut groups: std::collections::BTreeMap<i64, ScoreAgg> = std::collections::BTreeMap::new();
    for s in &stats {
        let uid = s.user_id.unwrap_or(0);
        let (fields, chars) = groups
            .entry(uid)
            .or_insert_with(|| (serde_json::Map::new(), serde_json::Map::new()));
        match s.content.as_ref().and_then(|c| c.get("fields")) {
            Some(serde_json::Value::Object(m)) => merge_int_map(fields, m),
            _ => {
                if let Some(v) = s
                    .content
                    .as_ref()
                    .and_then(|c| c.get("count"))
                    .and_then(|v| v.as_i64())
                {
                    let e = fields.entry("content".to_string()).or_insert(0.into());
                    *e = serde_json::Value::from(e.as_i64().unwrap_or(0) + v);
                }
            },
        }
        match s.content.as_ref().and_then(|c| c.get("chars")) {
            Some(serde_json::Value::Object(m)) => merge_int_map(chars, m),
            _ => {
                if let Some(v) = s
                    .content
                    .as_ref()
                    .and_then(|c| c.get("fieldWeight"))
                    .and_then(|v| v.as_f64())
                {
                    let e = chars.entry("content".to_string()).or_insert(0.into());
                    *e = serde_json::Value::from(e.as_i64().unwrap_or(0) + v as i64);
                }
            },
        }
    }

    // 批量查询用户基础信息（前端按 user.nickname/username 展示）
    let user_ids: Vec<i64> = groups.keys().copied().collect();
    let mut user_infos: std::collections::HashMap<i64, serde_json::Value> =
        std::collections::HashMap::new();
    if !user_ids.is_empty() {
        use sea_orm::{ColumnTrait, QueryFilter};
        let users = sys_user_model::Entity::find_safety()
            .filter(sys_user_model::Column::Id.is_in(&user_ids))
            .all(db)
            .await?;
        for u in users {
            user_infos.insert(
                u.id,
                serde_json::json!({
                    "username": u.username,
                    "nickname": u.nickname,
                }),
            );
        }
    }

    let list: Vec<serde_json::Value> = groups
        .into_iter()
        .map(|(uid, (fields, chars))| {
            serde_json::json!({
                "userId": uid,
                // 必须为对象：前端解构 `user.nickname` 时 null/undefined 会抛 TypeError
                "user": user_infos.get(&uid).cloned().unwrap_or_else(|| serde_json::json!({})),
                "scope": payload.scope,
                "span": payload.span,
                "data": { "chars": chars, "fields": fields },
            })
        })
        .collect();

    Ok(CommonResponse::new(Ok(serde_json::Value::Array(list))))
}

/// 将 src 中的整数条目累加合并进 target（跨时间桶同一用户的字段计数相加）。
fn merge_int_map(
    target: &mut serde_json::Map<String, serde_json::Value>,
    src: &serde_json::Map<String, serde_json::Value>,
) {
    for (k, v) in src {
        if let Some(n) = v.as_i64() {
            let e = target.entry(k.clone()).or_insert(0.into());
            *e = serde_json::Value::from(e.as_i64().unwrap_or(0) + n);
        }
    }
}

/// 将毫秒时间戳转换为 NaiveDateTime。
fn timestamp_to_naive(ms: f64) -> NaiveDateTime {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| Utc::now().naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_matches_reference_distances() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", "abd"), 1);
        assert_eq!(levenshtein("xyz", "xy"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("日本語", "日本"), 1);
    }

    #[test]
    fn normalized_field_coerces_numeric_strings_and_ints() {
        // Java ActionsPre：hiddenFlag/refreshTime 按整数、文本按字符串比较。
        assert_eq!(
            normalized_field("hiddenFlag", &serde_json::json!(1)),
            normalized_field("hiddenFlag", &serde_json::json!("1"))
        );
        assert_eq!(
            normalized_field("refreshTime", &serde_json::json!(100.0)),
            normalized_field("refreshTime", &serde_json::json!(100))
        );
        assert_ne!(
            normalized_field("hiddenFlag", &serde_json::json!(0)),
            normalized_field("hiddenFlag", &serde_json::json!(1))
        );
        assert_eq!(
            normalized_field("markerTitle", &serde_json::json!("a")),
            serde_json::json!("a")
        );
    }

    #[test]
    fn day_bucket_aligns_to_shanghai_calendar_day() {
        // 2026-10-22 01:00 UTC = 09:00 (+08) 当日；桶起点应为
        // 2026-10-21 16:00 UTC（= 当地 00:00）。
        let t = chrono::NaiveDate::from_ymd_opt(2026, 10, 22)
            .unwrap()
            .and_hms_opt(1, 0, 0)
            .unwrap();
        let bucket = day_bucket_utc(t);
        assert_eq!(
            bucket,
            chrono::NaiveDate::from_ymd_opt(2026, 10, 21)
                .unwrap()
                .and_hms_opt(16, 0, 0)
                .unwrap()
        );
        // 当地 23:59（UTC 15:59）与次日凌晨（UTC 16:01）分属两个桶。
        let late = chrono::NaiveDate::from_ymd_opt(2026, 10, 22)
            .unwrap()
            .and_hms_opt(15, 59, 0)
            .unwrap();
        let next = chrono::NaiveDate::from_ymd_opt(2026, 10, 22)
            .unwrap()
            .and_hms_opt(16, 1, 0)
            .unwrap();
        assert_eq!(day_bucket_utc(late), bucket);
        assert_ne!(day_bucket_utc(next), bucket);
    }
}
