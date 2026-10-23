//! Observability endpoints and HTTP access logging.
//!
//! Three pieces live here, all root-mounted next to `/.well-known/jwks.json`:
//!
//! - `GET /health` — liveness/readiness probe for orchestrators and load
//!   balancers. It runs `SELECT 1` against the configured Postgres pool and,
//!   when Redis is configured, a `PING`; both probes are bounded by
//!   [`PROBE_TIMEOUT`] so a dead dependency can never hang the probe. Redis
//!   is treated as optional (the backend degrades gracefully when it is
//!   unset), so its absence is reported as `"redis": null` rather than a
//!   failure.
//! - `GET /metrics` — Prometheus text exposition. Counters/histograms are
//!   emitted through the `metrics` facade and rendered by
//!   `metrics-exporter-prometheus`; the recorder is installed once per
//!   process by [`init_metrics`].
//! - [`access_log`] — a one-line access log plus the HTTP request counter and
//!   latency histogram. It is a plain axum middleware rather than a
//!   `tower_http::trace::TraceLayer` callback because `OnResponse` only
//!   receives `(&Response, Duration, &Span)` — the request (and therefore
//!   method/path) is gone by then, and this repository logs through
//!   `log`/env_logger, so span fields would never reach the log line. The
//!   middleware keeps method, path, status and duration on a single line and
//!   is mounted exactly once (outermost) to avoid duplicate logging.
//!
//! The recorder deliberately uses `metrics-exporter-prometheus` with
//! `default-features = false`: the default `push-gateway` feature pulls
//! `hyper-rustls/aws-lc-rs`, which would drag the aws-lc crypto provider into
//! the workspace and overrule its ring-only rustls policy. We only need the
//! recorder + text renderer, so the listener/push-gateway features stay off.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sea_orm::ConnectionTrait;
use serde_json::json;

use super::SharedDb;

/// Upper bound for a single dependency probe. Kept short so a dead database
/// reports `503` quickly instead of tying up orchestrator probes for the
/// pool's acquire timeout.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Process-wide Prometheus recorder. `metrics` allows a single global
/// recorder; the `OnceLock` makes installation idempotent for processes that
/// assemble the router more than once (integration tests) while still keeping
/// the handle available to `GET /metrics`.
static PROMETHEUS: OnceLock<Option<metrics_exporter_prometheus::PrometheusHandle>> =
    OnceLock::new();

/// Explicit latency buckets for the request-duration histogram. Without
/// buckets the exporter renders `histogram!` metrics as Prometheus summaries
/// (quantiles only); setting buckets makes `http_request_duration_seconds` a
/// real cumulative histogram with `_bucket`/`_sum`/`_count` series, which is
/// what latency SLOs and alerting rules consume. 5 ms – 10 s.
const HISTOGRAM_BUCKETS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Install the Prometheus recorder (no-op after the first call).
///
/// Called by `main` at startup; also called from [`super::router`] so tests
/// that assemble the router directly get a working `/metrics` endpoint
/// without booting the binary.
pub fn init_metrics() {
    PROMETHEUS.get_or_init(|| {
        let builder = match metrics_exporter_prometheus::PrometheusBuilder::new()
            .set_buckets(&HISTOGRAM_BUCKETS)
        {
            Ok(builder) => builder,
            Err(err) => {
                log::warn!("Prometheus histogram buckets rejected: {err}");
                return None;
            },
        };
        match builder.install_recorder() {
            Ok(handle) => Some(handle),
            Err(err) => {
                // A recorder can only be installed once per process; losing
                // that race must not take the service down (metrics are an
                // observability convenience, not a business dependency).
                log::warn!("Prometheus recorder not installed: {err}");
                None
            },
        }
    });
}

fn prometheus_handle() -> Option<&'static metrics_exporter_prometheus::PrometheusHandle> {
    PROMETHEUS.get().and_then(Option::as_ref)
}

/// `GET /metrics` — Prometheus text exposition (version 0.0.4).
///
/// Unauthenticated like `/health`: scrapers run inside the deployment
/// network and proxying a scrape through auth would require storing
/// credentials next to Prometheus.
pub async fn metrics() -> Response {
    match prometheus_handle() {
        Some(handle) => (
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            handle.render(),
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "metrics recorder unavailable\n",
        )
            .into_response(),
    }
}

/// `GET /health` — dependency-aware health probe.
///
/// Response shapes:
/// - DB down: `503 {"status":"unavailable","db":false,...}`
/// - Redis configured but down: `200 {"status":"degraded","db":true,"redis":false}`
/// - Redis not configured: `200 {"status":"ok","db":true,"redis":null}`
/// - all good: `200 {"status":"ok","db":true,"redis":true}`
pub async fn health(State(db): State<SharedDb>) -> Response {
    let db_probe = async { probe_db(&db.pg_conn).await };
    let redis_probe = async {
        match db.redis_conn.as_ref() {
            Some(client) => Some(probe_redis(client).await),
            None => None,
        }
    };
    let (db_ok, redis_state) = tokio::join!(db_probe, redis_probe);

    let (status_code, status) = if !db_ok {
        (StatusCode::SERVICE_UNAVAILABLE, "unavailable")
    } else if redis_state == Some(false) {
        (StatusCode::OK, "degraded")
    } else {
        (StatusCode::OK, "ok")
    };

    (
        status_code,
        axum::Json(json!({
            "status": status,
            "db": db_ok,
            "redis": redis_state,
        })),
    )
        .into_response()
}

/// `SELECT 1` inside the probe deadline. Any error (including the
/// mock-connection "no canned result" error used by HTTP-layer tests) is
/// reported as `false`; panics are impossible here because the probe only
/// awaits the pool's `query_one`.
async fn probe_db(pg: &sea_orm::DatabaseConnection) -> bool {
    matches!(
        tokio::time::timeout(PROBE_TIMEOUT, pg.execute_unprepared("SELECT 1")).await,
        Ok(Ok(_))
    )
}

/// `PING` inside the probe deadline; any connection/command error is
/// reported as `false`.
async fn probe_redis(client: &redis::Client) -> bool {
    let probe = async {
        match client.get_multiplexed_async_connection().await {
            Ok(mut conn) => redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .is_ok(),
            Err(_) => false,
        }
    };
    matches!(tokio::time::timeout(PROBE_TIMEOUT, probe).await, Ok(true))
}

/// One-line access log + HTTP metrics for every request.
///
/// Mounted outermost in [`super::router`] so the status it observes is the
/// final one (including `TimeoutLayer` 408s and fallback 501s), and mounted
/// exactly once so no other layer duplicates the line.
pub async fn access_log(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_owned();

    let response = next.run(request).await;
    let elapsed = started.elapsed();
    let status = response.status();

    // Metric labels stay low-cardinality: method + status class only. The raw
    // path is deliberately not a label (ids make it unbounded); it lives in
    // the log line below.
    let method_label = static_method_label(&method);
    metrics::counter!(
        "http_requests_total",
        "method" => method_label,
        // metrics 0.24 label values are strings; keep the exact status code so
        // dashboards can split 4xx/5xx without a second label.
        "status" => status.as_u16().to_string(),
    )
    .increment(1);
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method_label,
    )
    .record(elapsed.as_secs_f64());

    log::info!(
        target: "genshin_cloud::access",
        "{} {} -> {} in {}ms",
        method,
        path,
        status.as_u16(),
        elapsed.as_millis()
    );

    response
}

/// Bounded `method` label value ("OTHER" keeps cardinality fixed forever).
fn static_method_label(method: &axum::http::Method) -> &'static str {
    match *method {
        axum::http::Method::GET => "GET",
        axum::http::Method::POST => "POST",
        axum::http::Method::PUT => "PUT",
        axum::http::Method::DELETE => "DELETE",
        axum::http::Method::PATCH => "PATCH",
        axum::http::Method::HEAD => "HEAD",
        axum::http::Method::OPTIONS => "OPTIONS",
        _ => "OTHER",
    }
}
