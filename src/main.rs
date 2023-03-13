use tonsail_server::{
    configuration::get_configuration, routes::Application, util::tracing::initialize_tracing,
};

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    initialize_tracing().await;

    let config = get_configuration().expect("Failed to read configuration");

    let application = Application::build(config)
        .await
        .expect("Could not build the app");

    application.run_until_stopped().await
}
