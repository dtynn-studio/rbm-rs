use std::net::SocketAddr;

use crate::{
    proto::{Codec, ProtoCommand, Raw},
    Result,
};

mod transport;
pub mod v1;

pub use transport::Transport;

pub trait RawHandler<C: Codec> {
    // return if the handler is executed
    fn recv(&self, raw: &Raw<C>) -> Result<bool>;

    fn gc(&self) -> Result<()>;
}

pub trait Client<C: Codec>: Sized {
    fn connect<T: Transport + 'static>(
        bind: Option<SocketAddr>,
        dest: SocketAddr,
        host: C::Sender,
        target: C::Receiver,
    ) -> Result<Self>;

    fn send_cmd<CMD: ProtoCommand<C>>(
        &self,
        receiver: Option<C::Receiver>,
        cmd: CMD,
        need_ack: bool,
    ) -> Result<Option<CMD::Resp>>
    where
        CMD::Resp: Send + 'static;

    fn register_raw_handler<H: RawHandler<C> + Send + 'static>(
        &self,
        name: &str,
        hdl: H,
    ) -> Result<()>;

    fn unregister_raw_handler<H: RawHandler<C>>(&self, name: &str) -> Result<bool>;
}
