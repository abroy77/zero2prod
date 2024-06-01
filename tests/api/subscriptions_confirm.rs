use crate::helpers::spawn_app;
use reqwest;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    //Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    // email server in the test app is a mock server that records
    // requests that it received.
    // it would've received a request to send an email. with the
    // body of the email. which contains the confirmation link.
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(email_request);

    // make sure we don't call random APIs on the web
    assert_eq!(confirmation_links.html.host_str().unwrap(), "127.0.0.1");

    let response = reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=Abhishek%20Roy&email=royabhishek77%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;

    // retrieve the confirmation link
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(email_request);

    // Act
    reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute request.")
        .error_for_status()
        .unwrap();

    // Assert

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "royabhishek77@gmail.com");
    assert_eq!(saved.name, "Abhishek Roy");
    assert_eq!(saved.status, "confirmed");
}
