use axum::{routing::IntoMakeService, Router, Server};
use backon::{ExponentialBuilder, Retryable};
use configuration::Settings;
use fred::{pool::RedisPool, prelude::RedisError, types::RedisConfig};
use hyper::server::conn::AddrIncoming;
use prisma::PrismaClient;
use prisma_client_rust::NewClientError;
use routes::create_router;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tracing::{info, instrument};
use util::app_error::AppError;

pub mod configuration;
pub mod domain;
pub mod prisma;
pub mod routes;
pub mod util;

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
