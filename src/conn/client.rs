use super::transport::Transport;
use crate::{
    proto::{Codec, Message},
    Result,
};

pub trait Client<T, C, M>
where
    T: Transport,
    C: Codec,
    M: Message<Ident = C::CmdIdent>,
{
    fn request(&self, m: M) -> Result<Option<M::Response>>;
    fn send(&self, m: M) -> Result<()>;
}
