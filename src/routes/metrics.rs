use super::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use prisma_client_rust::chrono::NaiveDateTime;
use sea_query::{ColumnRef, Expr, Iden, PostgresQueryBuilder, Query as SeaQuery};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use tracing::{debug, instrument};

#[derive(Debug, Serialize)]
pub struct HttpMetric {
    name: String,
    run_id: String,
    scenario: String,
    url: String,
    method: String,
    status: String,
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
    return 10;
}

impl From<&Row> for HttpMetric {
    fn from(row: &Row) -> Self {
        Self {
            name: row.get("name"),
            run_id: row.get("runID"),
            scenario: row.get("scenario"),
            url: row.get("url"),
            method: row.get("method"),
            status: row.get("status"),
            ts: row.get("ts"),
            value: row.get("value"),
        }
    }
}

#[instrument(name = "Getting metrics", skip_all)]
pub async fn get_metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricQuery>,
) -> Json<Vec<HttpMetric>> {
    debug!("{:?}", params);
    let sql = SeaQuery::select()
        .columns([ColumnRef::Asterisk])
        .from(Metrics::Table)
        .and_where(Expr::col(Metrics::RunID).eq(params.run_id))
        .and_where(Expr::col(Metrics::Name).eq(params.name))
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
    debug!("{}", sql);
    let conn = state
        .pg_client
        .get()
        .await
        .expect("Could not get connection");
    let stmt = conn.prepare(&sql).await.expect("Could not prepare query");
    let rows = conn.query(&stmt, &[]).await.expect("Failed query");
    let metrics: Vec<HttpMetric> = rows.iter().map(|r| r.try_into().unwrap()).collect();
    Json(metrics)
}
