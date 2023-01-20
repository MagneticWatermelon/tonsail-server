use std::time::Duration;

use axum::{body::BoxBody, Router};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    AuthLayer,
};
use http::{Method, Response};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{Level, Span};

use super::{
    auth::{TonsailUser, TonsailUserStore},
    AppState,
};

pub fn add_cors_layer(router: Router<AppState>) -> Router<AppState> {
    router.layer(
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any),
    )
}

type TonsailAuthLayer = AuthLayer<TonsailUserStore, TonsailUser>;
pub fn add_auth_layer(router: Router<AppState>, state: AppState) -> Router<AppState> {
    let user_store = TonsailUserStore::new(state.db_client.clone());
    let auth_layer: TonsailAuthLayer = AuthLayer::new(user_store, &state.secret);
    router
        .layer(auth_layer)
        .layer(SessionLayer::new(MemoryStore::new(), &state.secret))
}

pub fn add_trace_layer(router: Router<AppState>) -> Router<AppState> {
    router.layer(
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
}
