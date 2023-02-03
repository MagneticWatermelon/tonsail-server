use self::auth::{login, register_new_user};
use self::layers::{add_auth_layer, add_cors_layer, add_trace_layer};
use self::metrics::get_metrics;
use self::organizations::{get_organizations, update_organization};
use self::project::{create_project, get_project, update_project};
use self::user::{get_user, update_user};
use crate::prisma::PrismaClient;
use axum::routing::{get, post};
use axum::Router;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use health_check::health_check;
use organizations::get_organization;
use rand::Rng;
use std::sync::Arc;
use tokio_postgres::NoTls;

pub mod auth;
pub mod health_check;
pub mod layers;
pub mod metrics;
pub mod organizations;
pub mod project;
pub mod user;

#[derive(Debug, Clone)]
pub struct AppState {
    db_client: Arc<PrismaClient>,
    pg_client: Pool<PostgresConnectionManager<NoTls>>,
    secret: [u8; 64],
}

impl AppState {
    pub fn new(client: PrismaClient, pg_client: Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self {
            db_client: Arc::new(client),
            pg_client,
            secret: rand::thread_rng().gen::<[u8; 64]>(),
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let mut app = Router::new()
        .route("/login", post(login))
        .route("/register", post(register_new_user))
        .route("/metrics", get(get_metrics))
        .route("/health_check", get(health_check))
        .route("/users/:user_id", get(get_user).put(update_user))
        .route("/projects", post(create_project))
        .route(
            "/projects/:project_id",
            get(get_project).put(update_project),
        )
        .route("/organizations", get(get_organizations))
        .route(
            "/organizations/:organization_id",
            get(get_organization).put(update_organization),
        );
    app = add_cors_layer(app);
    app = add_auth_layer(app, state.clone());
    app = add_trace_layer(app);
    app.with_state(state)
}
