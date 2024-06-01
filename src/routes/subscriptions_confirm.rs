use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(parameters, connection_pool)
)]
pub async fn confirm(parameters: Query<Parameters>, connection_pool: Data<PgPool>) -> HttpResponse {
    let subscriber_id =
        match get_subscriber_id_by_token(&parameters.subscription_token, &connection_pool).await {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };
    match subscriber_id {
        None => return HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if set_subscriber_status(subscriber_id, &connection_pool, "confirmed")
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError().finish();
            }
        }
    };

    HttpResponse::Ok().finish()
}

async fn get_subscriber_id_by_token(
    token: &str,
    connection_pool: &PgPool,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#" SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1 "#,
        token
    )
    .fetch_optional(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}

async fn set_subscriber_status(
    subscriber_id: Uuid,
    connection_pool: &PgPool,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = $1 WHERE id = $2"#,
        status,
        subscriber_id
    )
    .execute(connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
