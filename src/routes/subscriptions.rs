use std::{
    fmt::{Debug, Display},
    ops::DerefMut,
};

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Utc;
use lettre::transport::smtp;
use rand::{distr::Alphanumeric, rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FromData {
    name: String,
    email: String,
}

impl TryFrom<FromData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FromData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { name, email })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    form: web::Form<FromData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    // let subscriber_name = SubscriberName(&form.name);

    let new_subscriber = form.0.try_into()?;

    let mut transaction = pool.begin().await.context("Failed to acquire a Postgres connection from the pool")?;

    let subscribtion_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber in the database")?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, &subscribtion_id, &subscription_token).await.context("Failed to store subscription token in the database")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await.context("Failed to send a confirmation email")?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    // DatabaseError(sqlx::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}



impl From<String> for SubscribeError {
    fn from(error: String) -> Self {
        SubscribeError::ValidationError(error)
    }
}


impl Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}




impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            // SubscribeError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
           SubscribeError::UnexpectedError(_) => {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[tracing::instrument(
    name = "Storing subscription token in the database",
    skip(transaction, subscriber_id, token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
    token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query(
        r#"
        INSERT INTO subscription_tokens (subscription_id, subscription_token)
        VALUES ($1, $2)
    "#,
    )
    .bind(subscriber_id)
    .bind(token)
    .execute(transaction.deref_mut())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;
    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

// impl ResponseError for StoreTokenError {}
impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token."
        )
    }
}

impl Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //write!(f, "{}\nCaused by: \n\t{}", self, self.0)
        error_chain_fmt(self, f)
    }
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscriber_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscriber_token: &str,
) -> Result<(), smtp::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscriber_token
    );

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome to our newsletter!",
            format!(
                "<p><b>Thank you</b> for subscribing to our newsletter.<br/>Click to <a href=\"{}\">Confirm your subscription</a></p>", 
                confirmation_link
            ),
            format!(
                "Thank you for subscribing to our newsletter.\nClick to confirm your subscription: {}", 
                confirmation_link
            ),
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.name.as_ref(),
        new_subscriber.email.as_ref(),
        Utc::now()
    )
    .execute(transaction.deref_mut())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

fn error_chain_fmt(e: &dyn std::error::Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", e)?;
    if let Some(source) = e.source() {
        write!(f, "\nCaused by: ")?;
        error_chain_fmt(source, f)
    } else {
        Ok(())
    }
}
