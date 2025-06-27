use std::ops::Sub;

use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use fake::{
        faker::{
            internet::raw::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        locales::EN,
        Fake,
    };
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use super::*;
    
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail(EN).fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail(EN).fake()).unwrap();

        let subject: String = Sentence(1..2).fake();
        let content = Paragraph(1..10).fake::<String>();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
