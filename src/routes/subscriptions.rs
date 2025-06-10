use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FromData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FromData>, pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.name,
        form.email,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await{
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(e) => {
           println!("Failed to save subscription: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

}
