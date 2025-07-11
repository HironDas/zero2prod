use crate::helpers::spawn_app;

#[tokio::test]
async  fn confirmations_without_token_are_rejected_with_a_400() {
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
async fn the_link_returned_by_subscribe_returns_a_200_if_called(){
    // Arrange
    let app = spawn_app().await;
    let body = "name=Hiron%20Das&email=hcdas.09%40gmail.com";

    app.post_subscriptions(body.into())
        .await;


}