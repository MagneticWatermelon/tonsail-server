use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use fred::{pool::RedisPool, types::RedisConfig};
use tokio_postgres::NoTls;
use tonsail_server::{configuration::get_configuration, run, util::retry::try_build_prisma};
use tracing::{info, subscriber::set_global_default};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => {
            info!("No .env file found");
        }
    }
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(false)
        .with_line_number(false);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Tracing subscriber has been already set");

    let config = get_configuration().expect("Failed to read configuration");

    let client = try_build_prisma(3)
        .await
        .expect("Failed to get Prisma client");

    // set up connection pool for QuestDB
    let manager = PostgresConnectionManager::new_from_stringlike(&config.questdb.url, NoTls)
        .expect("Failed to create Postgres config");

    let pool = Pool::builder().build(manager).await.unwrap();

    // Redis pool creation
    let rds_config =
        RedisConfig::from_url(&config.redis.url).expect("Failed to create Redis config");
    let rds_pool = RedisPool::new(rds_config, None, None, 6).expect("Failed to create Redis pool");
    rds_pool.connect();
    rds_pool
        .wait_for_connect()
        .await
        .expect("Could not connect to Redis");

    let addr = config.application.address_string();

    run(&addr, client, rds_pool, pool, config.secret).await
}
