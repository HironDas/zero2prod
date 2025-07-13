use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", &app.address))
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=Hiron%20Das&email=hcdas.09%40gmail.com";
    let client = reqwest::Client::new();

    let mailhog_api_url = "http://localhost:8025/api/v1";

    // Delete All OLD Emails
    client
        .delete(&format!("{}/messages", mailhog_api_url))
        .send()
        .await
        .expect("Failed to delete old emails.");
    app.post_subscriptions(body.into()).await;

    let response = client
        .get(format!("{}/messages", mailhog_api_url))
        .send()
        .await
        .expect("Failed to fetch emails from mail server.");

    let confirmation_links = app.get_configuration_links(response).await;
    //assert
    assert_eq!(confirmation_links.html.host_str().unwrap(), "127.0.0.1");

    let mut confirmation_link = confirmation_links.html;
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link)
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=Hiron%20Das&email=hcdas.09%40gmail.com";
    let client = reqwest::Client::new();

    app.post_subscriptions(body.into()).await;

    let email_request = client
        .get(format!("{}/messages", "http://localhost:8025/api/v1"))
        .send()
        .await
        .expect("Failed to fetch emails from mail server.");

    let confirmation_links = app.get_configuration_links(email_request).await;
    let mut confirmation_link = confirmation_links.html;
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link)
        .await
        .expect("Failed to execute request.");

    //Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription details.");

    assert_eq!(saved.email, "hcdas.09@gmail.com");
    assert_eq!(saved.name, "Hiron Das");
    assert_eq!(saved.status, "confirmed");
}
