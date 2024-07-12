use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::{EmailClient, EmailClientError},
    startup::ApplicationBaseUrl,
};
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[derive(thiserror::Error)]
#[error("A database error occurred while trying to store the subscription token")]
pub struct StoreTokenError(#[source] sqlx::Error);

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{\n}", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by: \n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// #[derive(thiserror::Error)]
// pub enum SubscribeError {
//     #[error("{0}")]
//     ValidationError(String),
//     #[error("Failed to acquire a Postgres connection from the pool")]
//     PoolError(#[source] sqlx::Error),
//     #[error("Failed to insert new subscriber into database")]
//     InsertSubscriberError(#[source] sqlx::Error),
//     #[error("Failed to commit SQL transaction to store a new subscriber")]
//     TransactionCommitError(#[source] sqlx::Error),
//     #[error("Failed to store subscription token")]
//     StoreTokenError(#[from] StoreTokenError),
//     #[error("Failed to send confirmation email")]
//     SendEmailError(#[from] EmailClientError),
// }

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for StoreTokenError {}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{1}")]
    UnexpectedError(#[source] Box<dyn std::error::Error>, String),
}

fn generate_subscription_token() -> String {
    let rng = rand::thread_rng();

    rng.sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect()
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection_pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;

    let mut transaction = connection_pool.begin().await.map_err(|e| {
        SubscribeError::UnexpectedError(
            Box::new(e),
            "Failed to acquire a Postgres connection from the pool".into(),
        )
    })?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .map_err(|e| {
            SubscribeError::UnexpectedError(
                Box::new(e),
                "Failed to insert a new subscriber in the database.".into(),
            )
        })?;

    let subscription_token = store_token(&mut transaction, subscriber_id)
        .await
        .map_err(|e| {
            SubscribeError::UnexpectedError(
                Box::new(e),
                "Failed to store the confirmation token for a new subscriber.".into(),
            )
        })?;

    transaction.commit().await.map_err(|e| {
        SubscribeError::UnexpectedError(
            Box::new(e),
            "Failed to commit SQL transaction to store a new subscriber.".into(),
        )
    })?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .map_err(|e| {
        SubscribeError::UnexpectedError(Box::new(e), "Failed to send a confirmation email.".into())
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Sending confirmation email",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), EmailClientError> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );

    let plain_body = format!(
        "Welcome to Roy's newsletter!\nVisit {} to confirm your subscription",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to Roy's newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription",
        confirmation_link
    );

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Inserting new subscriber into db"
    skip(transaction,new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
        VALUES ($1, $2, $3, $4, 'pending_confirmation')"#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}
#[tracing::instrument(
    name = "Storing subscription token in db",
    skip(transaction, subscriber_id)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
) -> Result<String, StoreTokenError> {
    let subscription_token = generate_subscription_token();
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) 
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id,
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(subscription_token)
}
