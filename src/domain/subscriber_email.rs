//! src/domain/subscriber_email.rs
use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
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
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};
    use fake::{faker::internet::raw::SafeEmail, locales::EN, Fake};
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = SafeEmail(EN).fake();
        assert_ok!(SubscriberEmail::parse(email));
    }

    #[derive(Debug, Clone)]
    struct SubscriberEmailFixture(pub String);

    // prop_compose! {
    //     fn valid_email()
    //     (mut rng in any::<u64>().prop_map(|seed| StdRng::seed_from_u64(seed))) 
    //     (email in SafeEmail(EN).fake_with_rng(&mut rng)) -> SubscriberEmailFixture {
    //         SubscriberEmailFixture(email)
    //     }
    // }

    fn safe_email_strategy()-> impl Strategy<Value = SubscriberEmailFixture> {
        any::<u64>().prop_map(|seed| {
            let mut rng = StdRng::seed_from_u64(seed);
            let email = SafeEmail(EN).fake_with_rng(&mut rng);
            SubscriberEmailFixture(email)
        })
    }

    proptest! {
        #[test]
        fn valid_emails_are_parsed_successfully_proptest(email in safe_email_strategy()) {
            assert_ok!(SubscriberEmail::parse(email.0));
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "testemail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@testemail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
