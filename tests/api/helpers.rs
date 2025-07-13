use std::net::{SocketAddr, TcpListener};

use once_cell::sync::Lazy;
use reqwest::{Response, Url};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

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

pub struct ConfirmationLinks {
    pub html: Url,
    pub plain_text: Url,
}
pub struct TestApp {
    pub address: String,
    pub db_pool: sqlx::PgPool,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_configuration_links(&self, email_response: Response) -> ConfirmationLinks {
        let emails: Vec<serde_json::Value> = email_response
            .json()
            .await
            .expect("Failed to parse response body as JSON.");

        //println!("Emails: {:?}", emails);

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .map(|l| l.as_str().to_string())
                .collect();
            assert_eq!(links.len(), 2);
            links
        };
        let msg = emails[0]["Content"]["Body"].as_str().unwrap();
        let links = get_link(msg);

        ConfirmationLinks {
            html:  Url::parse(&links[1]).expect("Failed to parse confirmation link from email body"),
            plain_text:  Url::parse(&links[0]).expect("Failed to parse confirmation link from email body"),
        }
    }

}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    // println!("Listening on {:#?}", address.to_string());

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build server");

    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        port: application_port,
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
