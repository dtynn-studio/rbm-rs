use super::{Deserialize, Message};

pub trait Command: Message {
    type Response: std::fmt::Debug + Deserialize;
}
