use self::auth::{check_me, login, logout, register_new_user, TonsailUser};
use self::layers::{add_auth_layer, add_cors_layer, add_trace_layer};
use self::metrics::get_metrics;
use self::organizations::{get_organizations, update_organization};
use self::project::{create_project, get_project, update_project};
use self::test_run::{create_test_run, get_test_run};
use self::tests::{create_test, get_test};
use self::user::{get_user, update_user};
use crate::prisma::PrismaClient;
use axum::routing::{get, post};
use axum::Router;
use axum_login::RequireAuthorizationLayer;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use fred::pool::RedisPool;
use health_check::health_check;
use organizations::get_organization;
use std::sync::Arc;
use tokio_postgres::NoTls;

pub mod auth;
pub mod health_check;
pub mod layers;
pub mod metrics;
pub mod organizations;
pub mod project;
pub mod test_run;
pub mod tests;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    db_client: Arc<PrismaClient>,
    pg_client: Pool<PostgresConnectionManager<NoTls>>,
    rds_client: RedisPool,
    secret: Vec<u8>,
}

impl AppState {
    pub fn new(
        client: PrismaClient,
        rds_client: RedisPool,
        pg_client: Pool<PostgresConnectionManager<NoTls>>,
        secret: Vec<u8>,
    ) -> Self {
        println!("{:?}", secret);
        Self {
            db_client: Arc::new(client),
            pg_client,
            rds_client,
            secret,
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let mut app = Router::new()
        .route("/me", get(check_me))
        .route("/logout", post(logout))
        .route("/metrics", get(get_metrics))
        .route("/users/:user_id", get(get_user).put(update_user))
        .route("/runs/:run_id", get(get_test_run).post(create_test_run))
        .route("/tests", post(create_test))
        .route("/tests/:test_id", get(get_test))
        .route("/projects", post(create_project))
        .route(
            "/projects/:project_id",
            get(get_project).put(update_project),
        )
        .route("/organizations", get(get_organizations))
        .route(
            "/organizations/:organization_id",
            get(get_organization).put(update_organization),
        )
        .route_layer(RequireAuthorizationLayer::<TonsailUser>::login())
        .route("/login", post(login))
        .route("/register", post(register_new_user))
        .route("/health_check", get(health_check));
    app = add_cors_layer(app);
    app = add_auth_layer(app, state.clone());
    app = add_trace_layer(app);
    app.with_state(state)
}
