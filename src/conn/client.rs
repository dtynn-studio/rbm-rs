use super::transport::Transport;
use crate::{
    proto::{Codec, Msg},
    Result,
};

pub struct Client<T>
where
    T: Transport,
{
    tran: T,
}
