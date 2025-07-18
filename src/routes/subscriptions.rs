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

    let mut transaction = pool.begin().await.map_err(SubscribeError::PoolError)?;

    let subscribtion_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .map_err(SubscribeError::InsertSubscriberError)?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, &subscribtion_id, &subscription_token).await?;

    transaction
        .commit()
        .await
        .map_err(SubscribeError::TransactionCommitError)?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

pub enum SubscribeError {
    ValidationError(String),
    // DatabaseError(sqlx::Error),
    StoreTokenError(StoreTokenError),
    SendEmailError(smtp::Error),
    PoolError(sqlx::Error),
    InsertSubscriberError(sqlx::Error),
    TransactionCommitError(sqlx::Error),
}

impl From<smtp::Error> for SubscribeError {
    fn from(error: smtp::Error) -> Self {
        SubscribeError::SendEmailError(error)
    }
}

// impl From<sqlx::Error> for SubscribeError {
//     fn from(error: sqlx::Error) -> Self {
//         SubscribeError::DatabaseError(error)
//     }
// }

impl From<String> for SubscribeError {
    fn from(error: String) -> Self {
        SubscribeError::ValidationError(error)
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(error: StoreTokenError) -> Self {
        SubscribeError::StoreTokenError(error)
    }
}

impl Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscribeError::ValidationError(e) => write!(f, "Validation error: {}", e),
            // SubscribeError::DatabaseError(e) => write!(f, "Database error: {}", e),
            SubscribeError::StoreTokenError(_e) => write!(
                f,
                "Failed to store the confirmation token for a new subscriber."
            ),
            SubscribeError::SendEmailError(_e) => {
                write!(f, "Failed to send a confitrmation email.")
            }
            SubscribeError::PoolError(_) => write!(f, "Failed to get a connection from the pool."),
            SubscribeError::InsertSubscriberError(_e) => {
                write!(f, "Failed to insert a new subscriber into the database.")
            }
            SubscribeError::TransactionCommitError(_e) => {
                write!(f, "Failed to commit the transaction.")
            }
        }
    }
}

impl std::error::Error for SubscribeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SubscribeError::ValidationError(_) => None,
            // SubscribeError::DatabaseError(e) => Some(e),
            SubscribeError::StoreTokenError(e) => Some(e),
            SubscribeError::SendEmailError(e) => Some(e),
            SubscribeError::PoolError(e)
            | SubscribeError::InsertSubscriberError(e)
            | SubscribeError::TransactionCommitError(e) => Some(e),
        }
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            // SubscribeError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            SubscribeError::StoreTokenError(_)
            | SubscribeError::SendEmailError(_)
            | SubscribeError::PoolError(_)
            | SubscribeError::InsertSubscriberError(_)
            | SubscribeError::TransactionCommitError(_) => {
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
