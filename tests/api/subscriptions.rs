use fake::{faker::internet::raw::SafeEmail, locales, Fake};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    //let configuration = get_configuration().expect("Failed to read configuration");
    // let connection_string = configuration.database.connection_string().expose_secret();

    // let mut connection =
    //     PgConnection::connect(configuration.database.connection_string().expose_secret())
    //         .await
    //         .expect("Failed to connect to Postgres");

    // Act
    let body = "name=Hiron%20Das&email=hcdas.09%40gmail.com";

    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "hcdas.09@gmail.com");
    assert_eq!(saved.name, "Hiron Das");
}

#[tokio::test]
async fn subcribe_return_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
   
    let test_cases = vec![
        ("name=Hiron%20Das", "missing the email"),
        ("email=hcdas.09%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_return_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let app = spawn_app().await;
    
    let test_cases = vec![
        ("name=&email=hcdas.09%40gmail.com", "empty name"),
        ("name=Hiron%20Das&email=", "empty email"),
        (
            "name=&email=definately-not-an-email",
            "empty name and email",
        ),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return 200 OK when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn test_email_contents() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let mailhog_api_url = "http://localhost:8025/api/v1";

    // Delete All OLD Emails
    client
        .delete(&format!("{}/messages", mailhog_api_url))
        .send()
        .await
        .expect("Failed to delete old emails.");

    let fake_email = SafeEmail(locales::EN).fake::<String>();
    let body = format!("name=Hiron Das&email={}", fake_email);
    
    let response = app.post_subscriptions(body).await;

    assert_eq!(200, response.status().as_u16());

    let response = client
        .get(format!("{}/messages", mailhog_api_url))
        .send()
        .await
        .expect("Failed to fetch emails from mail server.");

    // let emails = response.json().await
    //     .expect("Failed to parse emails from response.");

    assert_eq!(200, response.status().as_u16());
}
