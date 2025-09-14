use anyhow::Result;

use _utils::jwt::AuthInfo;
use _utils::models::score::{ScoreDataRequest, ScoreGenerateRequest, ScoreResponse, ScoreSample};
use _utils::models::wrapper::CommonResponse;

fn span_to_seconds(span: &str) -> Option<f64> {
    match span {
        "hour" => Some(3600.0),
        "day" => Some(86400.0),
        s => s.parse::<f64>().ok(),
    }
}

fn deterministic_score(seed: u64) -> f64 {
    // 基于简单 LCG 的确定性伪随机，映射到 0.0..5.0
    let mut x = seed.wrapping_add(0x9E3779B97F4A7C15u64);
    x = x
        .wrapping_mul(6364136223846793005u64)
        .wrapping_add(1442695040888963407u64);
    let v = (x >> 33) as u32 as u64;
    let scaled = (v % 500) as f64 / 100.0; // 0.00 .. 4.99
    (scaled * 100.0).round() / 100.0
}

pub async fn do_generate_score(
    _auth: AuthInfo,
    payload: ScoreGenerateRequest,
) -> Result<CommonResponse<ScoreResponse>> {
    let start = payload.start_time;
    let end = payload.end_time;
    let span_sec = span_to_seconds(&payload.span).unwrap_or(3600.0);

    if end <= start || span_sec <= 0.0 {
        let payload = ScoreResponse {
            samples: Vec::new(),
            average: 0.0,
        };
        return Ok(CommonResponse::new(Ok(payload)));
    }

    let mut samples: Vec<ScoreSample> = Vec::new();
    let mut sum = 0.0f64;

    // 限制数据点数量以避免 OOM（内存不足）
    let max_points = 10_000usize;
    let mut i = 0usize;
    loop {
        let t = start + (i as f64) * span_sec;
        if t > end || samples.len() >= max_points {
            break;
        }

        // 种子由 scope 与时间戳派生以保证确定性
        let mut seed: u64 = payload
            .scope
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        seed = seed.wrapping_add(t as u64);

        let score = deterministic_score(seed);
        sum += score;
        samples.push(ScoreSample { time: t, score });
        i += 1;
    }

    let avg = if samples.is_empty() {
        0.0
    } else {
        sum / (samples.len() as f64)
    };

    let payload = ScoreResponse {
        samples,
        average: (avg * 100.0).round() / 100.0,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_get_score_data(
    _auth: AuthInfo,
    payload: ScoreDataRequest,
) -> Result<CommonResponse<ScoreResponse>> {
    // For now, return the same shape as generate; in future this can read cached/aggregated data
    do_generate_score(
        _auth,
        ScoreGenerateRequest {
            start_time: payload.start_time,
            end_time: payload.end_time,
            span: payload.span.clone(),
            scope: payload.scope.clone(),
        },
    )
    .await
}
