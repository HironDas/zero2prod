use std::ops::Sub;

use lettre::{message::{header::ContentType, Mailbox}, Address, Message, SmtpTransport, Transport};
//use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    smtp_client: SmtpTransport,
    // base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            smtp_client: SmtpTransport::builder_dangerous(base_url).build(),
           // base_url,
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
        let email = Message::builder()
            .from(Mailbox::new(
                Some("Hiron Das".to_string()),
                self.sender.as_ref().parse::<Address>().unwrap(),
            )).to(Mailbox { name: None, email: recipient.as_ref().parse::<Address>().unwrap() })
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(text_content.to_string())
            .unwrap();

        //let mailer = SmtpTransport::builder_dangerous("localhost:1025".to_string()).build();

        match self.smtp_client.send(&email){
            Ok(_)=> Ok(()),
            Err(e)=> Err(format!("Fail to send Email: {:?}", e))
        }
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
