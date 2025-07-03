use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::client::Tls,
    Address, Message, SmtpTransport, Transport,
};
//use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    smtp_client: SmtpTransport,
    // base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(host: String, port: u16, sender: SubscriberEmail) -> Self {
        Self {
            smtp_client: SmtpTransport::builder_dangerous(host.as_str())
                .port(port)
                .tls(Tls::None)
                // .credentials(credentials)
                // .authentication(vec![Mechanism::Plain])
                .build(),
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
            ))
            .to(Mailbox {
                name: None,
                email: recipient.as_ref().parse::<Address>().unwrap(),
            })
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_content.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_content.to_string()),
                    ),
            )
            .unwrap();

        //let mailer = SmtpTransport::builder_dangerous("localhost:1025".to_string()).build();

        match self.smtp_client.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Fail to send Email: {:?}", e)),
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use fake::{
//         faker::{
//             internet::raw::SafeEmail,
//             lorem::en::{Paragraph, Sentence},
//         },
//         locales::EN,
//         Fake,
//     };
//     use maik::MockServer;
//     //  use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

//     use super::*;

//     #[tokio::test]
//     async fn send_email_fires_a_request_to_base_url() {
//         let mock_server = MockServer::builder().no_verify_credentials().build(); //MockServer::start().await;
//         let sender = SubscriberEmail::parse(SafeEmail(EN).fake()).unwrap();
//         mock_server.start();

//         let email_client =
//             EmailClient::new(mock_server.host().to_string(), mock_server.port(), sender);

//         // Mock::given(any())
//         //     .respond_with(ResponseTemplate::new(200))
//         //     .expect(1)
//         //     .mount(&mock_server)
//         //     .await;

//         let subscriber_email = SubscriberEmail::parse(SafeEmail(EN).fake()).unwrap();

//         let subject: String = Sentence(1..2).fake();
//         let content = Paragraph(1..10).fake::<String>();

//         // Act
//         let _ = email_client
//             .send_email(subscriber_email, &subject, &content, &content)
//             .await;
//     }
// }
