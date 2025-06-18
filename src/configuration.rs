use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Debug, serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    #[allow(dead_code)]
    pub fn connection_string(&self) -> SecretString {
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    #[allow(dead_code)]
    pub fn connection_string_without_db(&self) -> SecretString {
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }

    pub fn without_db(&self)->PgConnectOptions {
        let ssl_mode = match self.require_ssl {
                true => PgSslMode::Require,
                false => PgSslMode::Prefer,
            };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let options = self
            .without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to get current directory");
    let config_directory = base_path.join("configuration");

    let settings = config::Config::builder().add_source(
        config::File::with_name(config_directory.join("base").to_str().unwrap()).required(true),
    );

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".to_string())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let settings = settings.add_source(
        config::File::with_name(
            config_directory
                .join(environment.as_str())
                .to_str()
                .unwrap(),
        )
        .required(true),
    );

    let settings = settings.add_source(
        config::Environment::with_prefix("APP")
            .prefix_separator("_")
            .separator("__"),
    );

    settings.build()?.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{} is not a valid environment. Use either `local` or `production`",
                other
            )),
        }
    }
}
