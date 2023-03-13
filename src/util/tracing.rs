use tracing::subscriber::set_global_default;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

pub async fn initialize_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(false)
        .with_line_number(false);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Tracing subscriber has been already set");
}
