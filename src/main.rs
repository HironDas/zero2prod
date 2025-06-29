//! src/main.rs
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let subscriber = get_subscriber(
        "zero2prod".into(),
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        std::io::stdout,
    );

    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let db_pool:PgPool = PgPoolOptions::new()
    .acquire_timeout(std::time::Duration::from_secs(2))
    .connect_lazy_with(configuration.database.with_db());
        // .expect("Failed to connect to Postgres");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    run(listener, db_pool.clone())?.await
}
