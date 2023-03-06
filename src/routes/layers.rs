use super::AppState;
use crate::{
    domain::auth::{TonsailUser, TonsailUserStore},
    util::redis_session_store::RedisSessionStore,
};
use axum::{body::BoxBody, Router};
use axum_login::{
    axum_sessions::{PersistencePolicy, SameSite, SessionLayer},
    AuthLayer,
};
use http::{Method, Request, Response};
use hyper::Body;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnFailure, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{Level, Span};

pub fn add_cors_layer(router: Router<AppState>) -> Router<AppState> {
    let origins = [
        "https://tonsail.dev".parse().unwrap(),
        "https://app.tonsail.dev".parse().unwrap(),
        "http://localhost:5173".parse().unwrap(),
    ];
    router.layer(
        CorsLayer::new()
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST, Method::PUT])
            .allow_origin(origins),
    )
}

type TonsailAuthLayer = AuthLayer<TonsailUserStore, TonsailUser>;
pub fn add_auth_layer(router: Router<AppState>, state: AppState) -> Router<AppState> {
    let user_store = TonsailUserStore::new(state.db_client.clone());

    let session_store =
        RedisSessionStore::from_pool(state.rds_client, Some("tonsail-session/".into()));

    let auth_layer: TonsailAuthLayer = AuthLayer::new(user_store, &state.secret);

    router.layer(auth_layer).layer(
        SessionLayer::new(session_store, &state.secret)
            .with_persistence_policy(PersistencePolicy::ExistingOnly)
            .with_cookie_name("api_sid")
            .with_same_site_policy(SameSite::Strict),
    )
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
                    .on_request(|_request: &Request<Body>, _span: &Span| {
                        tracing::info!("Started request")
                    })
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
