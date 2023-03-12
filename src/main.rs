use tonsail_server::{configuration::get_configuration, routes::Application};
use tracing::subscriber::set_global_default;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(false)
        .with_line_number(false);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Tracing subscriber has been already set");

    let config = get_configuration().expect("Failed to read configuration");

    let application = Application::build(config)
        .await
        .expect("Could not build the app");
    application.run_until_stopped().await
}
