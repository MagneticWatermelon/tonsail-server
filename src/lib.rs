use prisma::PrismaClient;
use routes::{create_router, AppState};
use std::{net::SocketAddr, str::FromStr};
use tracing::info;

pub mod configuration;
pub mod prisma;
pub mod routes;

pub async fn run(application_addr: &str, db_client: PrismaClient) {
    let state = AppState::new(db_client);
    let router = create_router(state);
    let addr = SocketAddr::from_str(application_addr).expect("Could not parse the address");
    let server = axum::Server::bind(&addr).serve(router.into_make_service());
    info!("Listening on {}", server.local_addr());
    server.await.unwrap()
}
