use crate::prisma::PrismaClient;
use axum::routing::post;
use axum::Router;
use axum::{body::BoxBody, routing::get};
use axum_login::axum_sessions::async_session::MemoryStore;
use axum_login::axum_sessions::SessionLayer;
use axum_login::AuthLayer;
use health_check::health_check;
use http::{Method, Response};
use organizations::get_organization;
use rand::Rng;
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::{
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{Level, Span};

use self::auth::{login, register_new_user, TonsailUser, TonsailUserStore};

pub mod auth;
pub mod health_check;
pub mod organizations;

#[derive(Debug, Clone)]
pub struct AppState {
    db_client: Arc<PrismaClient>,
    secret: [u8; 64],
}

type TonsailAuthLayer = AuthLayer<TonsailUserStore, TonsailUser>;

pub fn create_router(state: AppState) -> Router {
    let user_store = TonsailUserStore::new(state.db_client.clone());
    let auth_layer: TonsailAuthLayer = AuthLayer::new(user_store, &state.secret);
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register_new_user))
        .route("/health_check", get(health_check))
        .route("/organizations/:organization_id", get(get_organization))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        )
        .layer(auth_layer)
        .layer(SessionLayer::new(MemoryStore::new(), &state.secret))
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
            secret: rand::thread_rng().gen::<[u8; 64]>(),
        }
    }
}
