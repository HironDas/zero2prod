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
        todo!("Implement email sending logic here");
    }
}


#[cfg(test)]
mod tests{
    use super::*;
   #[tokio::test]
   async fn send_email_fires_a_request_to_base_url(){
    todo!("Implement a test that sends an email and checks the request to the base URL");
   }
}