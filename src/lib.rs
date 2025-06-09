pub mod configuration;
pub mod routes;
pub mod startup;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use std::net::TcpListener;


async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FromData{
    name: String,
    email: String,
}

async fn subscribe(_form: web::Form<FromData>)-> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_succeeds() {
        let response = health_check().await;
        assert!(response.status().is_success());
    }
}
