use reqwest;
// use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
// use zero2prod::startup::run;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
pub async fn subscribe_returns_200_when_valid_form() {
    // Arrange

    let addr = spawn_app();
    let settings = get_configuration().expect("Failed to read configuration.");
    let _connection_string = settings.database.connection_string();

    // let mut _connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();

    // Act
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
    //     .fetch_one(&mut connection)
    //     .await
    //     .expect("Failed to fetch saved subscription.");

    // assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    // assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_invalid_form() {
    // Arrange
    let addr = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = [
        ("name=Abhishek%20Roy", "missing email"),
        ("email=royabhishek77%40gmail.com", "missing name"),
        ("", "both missing"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &addr))
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to address");

    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
