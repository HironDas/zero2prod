use std::net::{SocketAddr, TcpListener};

use once_cell::sync::Lazy;
use sqlx::{Connection, PgConnection, PgPool, Executor};
use zero2prod::{configuration::{get_configuration, DatabaseSettings}, email_client::EmailClient, telemetry::{get_subscriber, init_subscriber}};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "info".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

// static EMAIL_CLIENT: Lazy<EmailClient>  = Lazy::new(|| {
//     let configuration = get_configuration().expect("Failed to read configuration");
//     let sender_email = configuration
//         .email_client
//         .sender()
//         .expect("Invalid Sender email address");
//     EmailClient::new(
//         configuration.email_client.host,
//         configuration.email_client.port,
//         sender_email,
//     )
// });
pub struct TestApp {
    pub address: String,
    pub db_pool: sqlx::PgPool,
}


pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address: SocketAddr = listener.local_addr().unwrap();
    let port = address.port();
    // println!("Listening on {:#?}", address.to_string());

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = format!("zero2prod_test_{}", port);
    let connection_pool = configure_database(&configuration.database).await;
    // Build a new email client

    // let email_client = *Lazy::force(&EMAIL_CLIENT);
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid Sender email address");
    let email_client = EmailClient::new(
        configuration.email_client.host,
        configuration.email_client.port,
        sender_email,
    );

    let server = zero2prod::startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}", port);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

// Test Isolation
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = sqlx::PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations");

    connection_pool
}
