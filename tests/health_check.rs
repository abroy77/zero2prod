use reqwest;
// use sqlx::{Connection, PgConnection};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
pub async fn subscribe_returns_200_when_valid_form() {
    // Arrange

    let test_app = spawn_app().await;
    let settings = get_configuration().expect("Failed to read configuration.");
    let _connection_string = settings.database.with_db();

    let client = reqwest::Client::new();

    // Act
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "royabhishek77@gmail.com");
    assert_eq!(saved.name, "Abhishek Roy");
}

#[tokio::test]
pub async fn subscribe_returns_400_when_invalid_form() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=Abhishek%20Roy", "missing email"),
        ("email=royabhishek77%40gmail.com", "missing name"),
        ("", "both missing"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // additional error message when failing on specific case
            "The API did not fail with 400 Bad request when the payload was {}",
            error_message
        );
    }
}

struct TestApp {
    address: String,
    connection_pool: sqlx::PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to address");

    let port = listener.local_addr().unwrap().port();

    let address = format!("http://127.0.0.1:{}", port);
    let mut settings = get_configuration().expect("Failed to read configuration.");
    // make random db name
    settings.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&settings.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    // create db
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate the db
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
