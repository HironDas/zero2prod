use crate::domain::SubscriberName;

pub struct NewSubscriber{
    pub name: SubscriberName,
    pub email: String,
}