//! src/domain/subscriber_email.rs
use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail{
    pub fn parse(s: String) -> Result<Self, String>{
        if <String as ValidateEmail>::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("Invalid subscriber email: {}", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod tests{
    use claim::{assert_err, assert_ok};
    use fake::{faker::internet::raw::SafeEmail, locales::EN, Fake};

    use super::*;

    #[test]
    fn valid_emails_are_parsed_successfully(){
        let email = SafeEmail(EN).fake();
        assert_ok!(SubscriberEmail::parse(email));
    }

    #[test]
    fn empty_string_is_rejected(){
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected(){
        let email = "testemail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected(){
        let email = "@testemail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    } 

}   