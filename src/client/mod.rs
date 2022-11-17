use std::net::SocketAddr;

use crate::{proto::Codec, Result};

mod transport;

pub trait RawMsgHandler {}

pub trait Client<C: Codec>: Sized {
    fn connect(
        bind: Option<SocketAddr>,
        dest: SocketAddr,
        host: C::Sender,
        target: C::Receiver,
    ) -> Result<Self>;
}
