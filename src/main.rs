use tonsail_server::{configuration::get_configuration, prisma::PrismaClient, run};
use tracing::subscriber::set_global_default;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Could not find a .env file");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(false)
        .with_line_number(false);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Tracing subscriber has been already set");

    let config = get_configuration().expect("Failed to read configuration");

    let client = PrismaClient::_builder()
        .build()
        .await
        .expect("Failed to get Prisma client");

    let addr = config.application.address_string();

    run(&addr, client).await
}
