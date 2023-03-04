use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use fred::pool::RedisPool;
use prisma::PrismaClient;
use routes::{create_router, AppState};
use std::{net::SocketAddr, str::FromStr};
use tokio_postgres::NoTls;
use tracing::info;

pub mod configuration;
pub mod domain;
pub mod prisma;
pub mod routes;
pub mod util;

pub async fn run(
    application_addr: &str,
    db_client: PrismaClient,
    rds_pool: RedisPool,
    pg_pool: Pool<PostgresConnectionManager<NoTls>>,
    secret: Vec<u8>,
) {
    let state = AppState::new(db_client, rds_pool, pg_pool, secret);
    let router = create_router(state);
    let addr = SocketAddr::from_str(application_addr).expect("Could not parse the address");
    let server = axum::Server::bind(&addr).serve(router.into_make_service());
    info!("Listening on {}", server.local_addr());
    server.await.unwrap()
}
