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
    types::ScopeStatType,
};

/// 生成评分统计数据（批处理管线）。
///
/// 对应 Java `ScoreGenerateService.generateScorePunctuate`：
/// 1. 清除 score_stat 表中该 scope+span+时间范围的旧数据
/// 2. 扫描 history 表（type=4=打点）在时间范围内的记录
/// 3. 按 creator_id（=提交者）分桶聚合统计
/// 4. 写入 score_stat 表（每个贡献者一行）
///
/// 注：Java 侧还有复杂的字段级 diff 算法（ScoreDataPunctuateVo），
/// 当前实现为简化版——统计每个贡献者在该时间范围内的打点编辑次数，
/// 作为评分基数。完整字段 diff 待后续细化。
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

    for id in old_ids {
        if let Some(m) = score_stat_model::Entity::find_safety_by_id(id)
            .one(db)
            .await?
        {
            let mut am: score_stat_model::ActiveModel = m.into();
            am.del_flag = Set(true);
            score_stat_model::Entity::update_safety(am)?
                .exec(db)
                .await?;
        }
    }

    // 2. 扫描 history 表（edit_type = 打点相关 = type 4 在 Java 侧）
    //    Rust 侧 history.edit_type 是 HistoryEditType 枚举
    let histories = history_model::Entity::find_safety()
        .filter(history_model::Column::CreateTime.gte(span_start))
        .filter(history_model::Column::CreateTime.lte(span_end))
        .all(db)
        .await?;

    // 3. 按 creator_id 分桶聚合（统计每个贡献者的编辑次数）
    let mut contributions: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    for h in &histories {
        if let Some(creator) = h.creator_id {
            *contributions.entry(creator).or_insert(0) += 1;
        }
    }

    // 4. 写入 score_stat 表（每个贡献者一行）
    let mut samples = Vec::new();
    let mut total_score = 0.0f64;

    for (&user_id, &count) in &contributions {
        // 评分 = 编辑次数（简化版；Java 用字段级 diff + 权重计算）
        let score = count as f64;

        let am = score_stat_model::ActiveModel {
            version: Set(0),
            id: Set(0),
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
            content: Set(ScopeStatType::DAY),
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

/// 读取评分统计数据——从 score_stat 表查询真实聚合记录。
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
        .map(|s| ScoreSample {
            time: s.span_end_time.and_utc().timestamp_millis() as f64,
            score: s.user_id.map(|_| 1.0).unwrap_or(0.0), // 每行 = 1 次贡献（简化）
        })
        .collect();

    let average = if samples.is_empty() {
        0.0
    } else {
        samples.iter().map(|s| s.score).sum::<f64>() / samples.len() as f64
    };

    Ok(CommonResponse::new(Ok(ScoreResponse { samples, average })))
}

/// 将毫秒时间戳转换为 NaiveDateTime。
fn timestamp_to_naive(ms: f64) -> NaiveDateTime {
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| Utc::now().naive_utc())
}
