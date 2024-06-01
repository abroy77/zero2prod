use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
pub async fn subscribe_returns_200_when_valid_form() {
    // Arrange

    let test_app = spawn_app().await;
    // Act
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
pub async fn subscribe_persists_the_new_subscriber() {
    // Arrange

    let test_app = spawn_app().await;
    // Act
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let _ = test_app.post_subscriptions(body.into()).await;

    // Assert

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "royabhishek77@gmail.com");
    assert_eq!(saved.name, "Abhishek Roy");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
pub async fn subscribe_returns_400_when_fields_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = [
        ("name=Abhishek%20Roy", "missing email"),
        ("email=royabhishek77%40gmail.com", "missing name"),
        ("", "both missing"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(invalid_body.into()).await;

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

#[tokio::test]
pub async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = [
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula%20Le%20Guin&email=", "empty email"),
        ("name=Ursula&email=not-an-email", "invalid email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(invalid_body.into()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // additional error message when failing on specific case
            "The API did not return a 400 Bad Request when payload when the payload was {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    //Mock asserts on drop
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert

    // get intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}
