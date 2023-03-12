use backon::ExponentialBuilder;
use backon::Retryable;
use tonsail_server::configuration::get_configuration;
use tonsail_server::{run, try_connect_postgres, try_connect_prisma, try_connect_redis};
use tracing::subscriber::set_global_default;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(false)
        .with_line_number(false);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Tracing subscriber has been already set");

    let config = get_configuration().expect("Failed to read configuration");

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

    let addr = config.application.address_string();

    run(&addr, prisma_client, rds_pool, pg_pool, config.secret).await
}
