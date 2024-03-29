use config::Config;
use serde::Deserialize;
use serde_aux::prelude::{deserialize_number_from_string, deserialize_vec_from_string_or_vec};
use tracing::info;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub questdb: QuestDBSettings,
    pub application: ApplicationSettings,
    #[serde(deserialize_with = "deserialize_vec_from_string_or_vec")]
    pub secret: Vec<u8>,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Deserialize)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Deserialize)]
pub struct QuestDBSettings {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        self.url.clone()
    }
}

impl ApplicationSettings {
    pub fn address_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let config_dir = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    match environment {
        Environment::Production => {
            if dotenvy::dotenv().is_err() {
                info!("No .env is found");
            }
        }
        _ => {
            if dotenvy::from_filename(".local.env").is_err() {
                info!("No .local.env is found");
            }
        }
    }

    let settings = Config::builder()
        .add_source(config::File::from(config_dir.join("base")).required(true))
        .add_source(config::File::from(config_dir.join(environment.as_str())).required(true))
        //E.g 'APPLICATION_PORT=5001 would set 'Settings.application.port'
        .add_source(config::Environment::with_prefix("APP").separator("_"))
        .add_source(config::Environment::with_prefix("DB").separator("_"))
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
    Test,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            "test" => Ok(Self::Test),
            other => Err(format!(
                "{other} is not a supported environment. Use either 'local or 'production",
            )),
        }
    }
}
