mod health_check;
pub mod organizations;

use crate::prisma::PrismaClient;
use axum::Router;
use axum::{body::BoxBody, routing::get};
use health_check::health_check;
use http::Response;
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{Level, Span};

use self::organizations::get_organization;

#[derive(Debug, Clone)]
pub struct AppState {
    db_client: Arc<PrismaClient>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/organizations/:organization_id", get(get_organization))
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .level(Level::INFO)
                                .include_headers(true),
                        )
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            |response: &Response<BoxBody>, latency: Duration, _span: &Span| {
                                tracing::info!(
                                    "Finished request latency={:?} status={}",
                                    latency,
                                    response.status()
                                )
                            },
                        )
                        .on_failure(DefaultOnFailure::new()),
                )
                .propagate_x_request_id(),
        )
        .with_state(state)
}

impl AppState {
    pub fn new(client: PrismaClient) -> Self {
        Self {
            db_client: Arc::new(client),
        }
    }
}
