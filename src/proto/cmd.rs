use super::{Deserialize, Message};

pub trait Command: Message + std::fmt::Debug {
    type Response: std::fmt::Debug + Deserialize;
}
