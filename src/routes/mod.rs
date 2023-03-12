use self::auth::{check_me, login, logout, register_new_user};
use self::layers::{add_auth_layer, add_cors_layer, add_trace_layer};
use self::metrics::get_metrics;
use self::organizations::{get_organizations, update_organization};
use self::project::{create_project, get_project, update_project};
use self::test_run::{create_test_run, get_test_run};
use self::tests::{create_test, get_test};
use self::user::{get_user, update_password, update_user};
use crate::configuration::Settings;
use crate::domain::auth::TonsailUser;
use crate::prisma::PrismaClient;
use crate::util::app_error::AppError;
use axum::routing::{get, post, put, IntoMakeService};
use axum::{Router, Server};
use axum_login::RequireAuthorizationLayer;
use backon::{ExponentialBuilder, Retryable};
use fred::pool::RedisPool;
use fred::prelude::RedisError;
use fred::types::RedisConfig;
use health_check::health_check;
use hyper::server::conn::AddrIncoming;
use organizations::get_organization;
use prisma_client_rust::NewClientError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, instrument};

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
    pg_client: Pool<Postgres>,
    rds_client: RedisPool,
    secret: Vec<u8>,
}
impl AppState {
    fn new(
        client: PrismaClient,
        rds_client: RedisPool,
        pg_client: Pool<Postgres>,
        secret: Vec<u8>,
    ) -> Self {
        Self {
            db_client: Arc::new(client),
            pg_client,
            rds_client,
            secret,
        }
    }
}

pub struct Application {
    pub server: Server<AddrIncoming, IntoMakeService<Router>>,
    pub router: Router,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, AppError> {
        let prisma_client = try_connect_prisma
            .retry(&ExponentialBuilder::default())
            .await
            .expect("Failed to get Prisma client");

        let pg_pool = { || try_connect_postgres(&config.questdb.url) }
            .retry(&ExponentialBuilder::default())
            .await
            .expect("Could not connect to Postgres");

        let rds_pool = { || try_connect_redis(&config.redis.url) }
            .retry(&ExponentialBuilder::default())
            .await
            .expect("Could not connect to Redis");

        let app_addr = config.application.address_string();
        let state = AppState::new(prisma_client, rds_pool, pg_pool, config.secret);
        let router = create_router(state);

        let addr = SocketAddr::from_str(&app_addr).expect("Could not parse the address");
        let server = axum::Server::bind(&addr).serve(router.clone().into_make_service());
        Ok(Self { server, router })
    }

    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        info!("Listening on {}", self.server.local_addr());
        self.server.await
    }
}

#[instrument(name = "Connecting Prisma")]
async fn try_connect_prisma() -> Result<PrismaClient, NewClientError> {
    PrismaClient::_builder().build().await
}

#[instrument(name = "Connecting Postgres")]
async fn try_connect_redis(url: &str) -> Result<RedisPool, RedisError> {
    // Redis pool creation
    let rds_config = RedisConfig::from_url(url)?;
    let rds_pool = RedisPool::new(rds_config, None, None, 6)?;
    rds_pool.connect();
    rds_pool.wait_for_connect().await?;
    Ok(rds_pool)
}

#[instrument(name = "Connecting Redis")]
async fn try_connect_postgres(url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    // set up connection pool for QuestDB
    PgPoolOptions::new().max_connections(10).connect(url).await
}
pub fn create_router(state: AppState) -> Router {
    let mut app = Router::new()
        .route("/me", get(check_me))
        .route("/logout", post(logout))
        .route("/metrics", get(get_metrics))
        .route("/users/:user_id", get(get_user).put(update_user))
        .route("/users/:user_id/password", put(update_password))
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
