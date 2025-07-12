use reqwest::Url;

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
    let emails: Vec<serde_json::Value> = response
        .json()
        .await
        .expect("Failed to parse response body as JSON.");

    //println!("Emails: {:?}", emails);

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .map(|l| l.as_str().to_string())
            .collect();
        assert_eq!(links.len(), 2);
        links
    };
    let msg = emails[0]["Content"]["Body"].as_str().unwrap();
    let links = get_link(msg);

    let confirmation_link = Url::parse(&links[1])
        .expect("Failed to parse confirmation link from email body");
    //assert
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    let response = reqwest::get(confirmation_link)
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());
}
