use super::AppState;
use crate::util::app_error::AppError;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use prisma_client_rust::chrono::NaiveDateTime;
use sea_query::{ColumnRef, Expr, Iden, PostgresQueryBuilder, Query as SeaQuery};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

#[derive(Debug, Serialize, FromRow)]
pub struct HttpMetric {
    name: String,
    #[sqlx(rename = "runID")]
    run_id: String,
    scenario: String,
    url: String,
    method: String,
    status: String,
    ts: NaiveDateTime,
    value: f32,
}

#[derive(Debug, Serialize)]
struct JSONMetric {
    name: String,
    run_id: String,
    values: Vec<TimeMetric>,
}

#[derive(Debug, Serialize)]
struct TimeMetric {
    ts: NaiveDateTime,
    value: f32,
}

#[derive(Iden)]
enum Metrics {
    Table,
    Name,
    #[iden = "runID"]
    RunID,
    Scenario,
    Url,
    Method,
    Status,
}

#[derive(Debug, Deserialize)]
pub struct MetricQuery {
    #[serde(rename(deserialize = "runID"))]
    run_id: String,
    name: String,
    scenario: Option<String>,
    url: Option<String>,
    method: Option<String>,
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: u64,
}

fn default_limit() -> u64 {
    10
}

#[instrument(name = "Getting metrics", skip_all)]
pub async fn get_metrics_catalog(State(state): State<AppState>) -> Result<Response, AppError> {
    let catalog = state
        .db_client
        .metrics_catalog()
        .find_many(vec![])
        .exec()
        .await?;

    Ok(Json(catalog).into_response())
}

#[instrument(name = "Getting metrics", skip_all)]
pub async fn get_metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricQuery>,
) -> Result<Response, AppError> {
    let sql = SeaQuery::select()
        .columns([ColumnRef::Asterisk])
        .from(Metrics::Table)
        .and_where(Expr::col(Metrics::RunID).eq(params.run_id.clone()))
        .and_where(Expr::col(Metrics::Name).eq(params.name.clone()))
        .conditions(
            params.scenario.is_some(),
            |q| {
                q.and_where(Expr::col(Metrics::Scenario).eq(params.scenario.unwrap()));
            },
            |_| {},
        )
        .conditions(
            params.url.is_some(),
            |q| {
                q.and_where(Expr::col(Metrics::Url).eq(params.url.unwrap()));
            },
            |_| {},
        )
        .conditions(
            params.method.is_some(),
            |q| {
                q.and_where(Expr::col(Metrics::Method).eq(params.method.unwrap()));
            },
            |_| {},
        )
        .conditions(
            params.status.is_some(),
            |q| {
                q.and_where(Expr::col(Metrics::Status).eq(params.status.unwrap()));
            },
            |_| {},
        )
        .limit(params.limit)
        .to_string(PostgresQueryBuilder);

    let metrics: Vec<HttpMetric> = sqlx::query_as(&sql).fetch_all(&state.pg_client).await?;
    let values: Vec<TimeMetric> = metrics
        .iter()
        .map(|m| TimeMetric {
            ts: m.ts,
            value: m.value,
        })
        .collect();
    let res = JSONMetric {
        name: params.name,
        run_id: params.run_id,
        values,
    };
    Ok(Json(res).into_response())
}
