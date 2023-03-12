use fred::{pool::RedisPool, prelude::RedisError, types::RedisConfig};
use prisma::PrismaClient;
use prisma_client_rust::NewClientError;
use routes::{create_router, AppState};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{net::SocketAddr, str::FromStr};
use tracing::{info, instrument};

pub mod configuration;
pub mod domain;
pub mod prisma;
pub mod routes;
pub mod util;

pub async fn run(
    application_addr: &str,
    db_client: PrismaClient,
    rds_pool: RedisPool,
    pg_pool: Pool<Postgres>,
    secret: Vec<u8>,
) {
    let state = AppState::new(db_client, rds_pool, pg_pool, secret);
    let router = create_router(state);
    let addr = SocketAddr::from_str(application_addr).expect("Could not parse the address");
    let server = axum::Server::bind(&addr).serve(router.into_make_service());
    info!("Listening on {}", server.local_addr());
    server.await.unwrap()
}

#[instrument(name = "Connecting Prisma")]
pub async fn try_connect_prisma() -> Result<PrismaClient, NewClientError> {
    PrismaClient::_builder().build().await
}

#[instrument(name = "Connecting Postgres")]
pub async fn try_connect_redis(url: &str) -> Result<RedisPool, RedisError> {
    // Redis pool creation
    let rds_config = RedisConfig::from_url(url)?;
    let rds_pool = RedisPool::new(rds_config, None, None, 6)?;
    rds_pool.connect();
    rds_pool.wait_for_connect().await?;
    Ok(rds_pool)
}

#[instrument(name = "Connecting Redis")]
pub async fn try_connect_postgres(url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    // set up connection pool for QuestDB
    PgPoolOptions::new().max_connections(10).connect(url).await
}
