use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QuerySelect, prelude::*};

use _database::{
    DB_CONN,
    models::common::{history as history_model, score_stat as score_stat_model},
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::score::{ScoreDataRequest, ScoreGenerateRequest, ScoreResponse, ScoreSample},
    models::wrapper::CommonResponse,
    types::{HistoryEditType, HistoryOperationType},
};

/// 生成评分统计数据（批处理管线）。
///
/// 对应 Java `ScoreGenerateService.generateScorePunctuate`：
/// 1. 清除 score_stat 表中该 scope+span+时间范围的旧数据
/// 2. 扫描 history 表（type=4=打点）在时间范围内的记录
/// 3. 按 creator_id（=提交者）分桶，按「字段级改动数」加权聚合
/// 4. 写入 score_stat 表（每个贡献者一行）
///
/// 字段级加权（对齐 Java `ScoreDataPunctuateVo` 的语义）：每条打点记录的
/// 权重 = 其 content JSON 的顶层字段数（Added / Modified 按字段数计，
/// Deleted 计 1，content 无法解析时按 1 计）。相比旧的「每条计 1」，
/// 改动字段越多的贡献得分越高。
pub async fn do_generate_score(
    _auth: AuthInfo,
    payload: ScoreGenerateRequest,
) -> Result<CommonResponse<ScoreResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    // 解析时间范围
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

    // 批量软删旧行（避免逐条 find+update 的 N+1 往返）
    if !old_ids.is_empty() {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        score_stat_model::Entity::update_many()
            .col_expr(
                score_stat_model::Column::DelFlag,
                sea_orm::sea_query::Expr::value(true),
            )
            .filter(score_stat_model::Column::Id.is_in(old_ids))
            .exec(db)
            .await?;
    }

    // 2. 扫描 history 表（type = 4 = 打点/点位，对齐 Java 侧）
    let histories = history_model::Entity::find_safety()
        .filter(history_model::Column::HistoryType.eq(HistoryOperationType::Position))
        .filter(history_model::Column::CreateTime.gte(span_start))
        .filter(history_model::Column::CreateTime.lte(span_end))
        .all(db)
        .await?;

    // 3. 按 creator_id 分桶，按字段级权重聚合
    let mut contributions: std::collections::HashMap<i64, (i64, f64)> =
        std::collections::HashMap::new();
    for h in &histories {
        if let Some(creator) = h.creator_id {
            let entry = contributions.entry(creator).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += entry_weight(h);
        }
    }

    // 4. 写入 score_stat 表（每个贡献者一行）
    let mut samples = Vec::new();
    let mut total_score = 0.0f64;

    for (&user_id, &(count, field_weight)) in &contributions {
        let score = field_weight;

        let am = score_stat_model::ActiveModel {
            version: Set(0),
            id: sea_orm::ActiveValue::NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(Some(user_id)),
            updater_id: Set(None),
            del_flag: Set(false),
            scope: Set(scope.clone()),
            span: Set(span_name.clone()),
            span_start_time: Set(span_start),
            span_end_time: Set(span_end),
            user_id: Set(Some(user_id)),
            content: Set(Some(serde_json::json!({
                "type": "DAY",
                "count": count,
                "fieldWeight": score,
            }))),
        };
        score_stat_model::Entity::insert(am).exec(db).await?;

        samples.push(ScoreSample {
            time: span_end.and_utc().timestamp_millis() as f64,
            score,
        });
        total_score += score;
    }

    let average = if samples.is_empty() {
        0.0
    } else {
        total_score / samples.len() as f64
    };

    Ok(CommonResponse::new(Ok(ScoreResponse { samples, average })))
}

/// 单条打点记录的字段级权重（Java `ScoreDataPunctuateVo` 语义的近似）。
///
/// - Added / Modified：content JSON 的顶层字段数（改动规模越大分越高）
/// - Deleted：固定 1（删除动作本身）
/// - content 无法解析：按 1 计
fn entry_weight(h: &history_model::Model) -> f64 {
    if h.edit_type == HistoryEditType::Deleted {
        return 1.0;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&h.content) else {
        return 1.0;
    };
    match value {
        serde_json::Value::Object(map) => map.len().max(1) as f64,
        _ => 1.0,
    }
}

/// 读取评分统计数据——从 score_stat 表查询真实聚合记录。
/// score 从每行的 content JSON 中读取（`fieldWeight`，旧数据回退到 `count`），
/// 不再固定为 1.0。
pub async fn do_get_score_data(
    _auth: AuthInfo,
    payload: ScoreDataRequest,
) -> Result<CommonResponse<ScoreResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let start = timestamp_to_naive(payload.start_time);
    let end = timestamp_to_naive(payload.end_time);

    let query = score_stat_model::Entity::find_safety()
        .filter(score_stat_model::Column::Scope.eq(&payload.scope))
        .filter(score_stat_model::Column::Span.eq(&payload.span))
        .filter(score_stat_model::Column::SpanStartTime.gte(start))
        .filter(score_stat_model::Column::SpanEndTime.lte(end))
        .limit(10_000);

    let stats = query.all(db).await?;

    let samples: Vec<ScoreSample> = stats
        .iter()
        .map(|s| {
            let score = score_from_content(s.content.as_ref());
            ScoreSample {
                time: s.span_end_time.and_utc().timestamp_millis() as f64,
                score,
            }
        })
        .collect();

    let average = if samples.is_empty() {
        0.0
    } else {
        samples.iter().map(|s| s.score).sum::<f64>() / samples.len() as f64
    };

    Ok(CommonResponse::new(Ok(ScoreResponse { samples, average })))
}

/// 从 score_stat.content JSON 提取分数：优先 `fieldWeight`（字段级加权），
/// 旧数据回退到 `count`（编辑次数），都缺失时按 0。
fn score_from_content(content: Option<&serde_json::Value>) -> f64 {
    content
        .and_then(|c| c.get("fieldWeight"))
        .and_then(|v| v.as_f64())
        .or_else(|| {
            content
                .and_then(|c| c.get("count"))
                .and_then(|v| v.as_f64())
        })
        .unwrap_or(0.0)
}

/// 将毫秒时间戳转换为 NaiveDateTime。
fn timestamp_to_naive(ms: f64) -> NaiveDateTime {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| Utc::now().naive_utc())
}
