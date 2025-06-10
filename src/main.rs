//! src/main.rs
use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::run};
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    // println!("Configuration: {:?}", configuration);
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    let configuration = get_configuration().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&configuration.database.connection_string()).await.expect("Failed to connect to Postgres");
    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    run(listener, db_pool.clone())?.await
}
