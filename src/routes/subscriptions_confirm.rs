use actix_web::{HttpResponse, web};
use sqlx::{pool, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirming a pending subscriber",
   skip(parameters, pool)
)]
pub async fn confirm(parameters: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let id = match get_subscriber_id_from_token(&pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to confirm subscription: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match id {
        Some(subscriber_id) => {
            if confirm_subscriber(&pool, subscriber_id).await.is_err() {
                tracing::error!("Failed to confirm subscriber with ID: {}", subscriber_id);
                return HttpResponse::InternalServerError().finish();
            }
        }
        None => {
            tracing::warn!("No subscriber found for token: {}", parameters.subscription_token);
            return HttpResponse::NotFound().finish();
        }
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(pool, subscriber_id)
)]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Retrieving subscriber ID from token",
    skip(pool)
)]
async fn get_subscriber_id_from_token(pool: &PgPool, token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscription_id FROM subscription_tokens WHERE subscription_token = $1"#,
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscription_id))
}