use std::net::SocketAddr;

use crate::{
    proto::{Codec, Command, Raw},
    Result,
};

mod transport;

pub trait RawHandler<C: Codec> {
    // return if the handler is finished
    fn recv(&mut self, raw: &Raw<C>) -> Result<bool>;
}

pub trait Client<C: Codec>: Sized {
    fn connect(
        bind: Option<SocketAddr>,
        dest: SocketAddr,
        host: C::Sender,
        target: C::Receiver,
    ) -> Result<Self>;

    fn send_cmd<CMD: Command<C>>(
        &mut self,
        receiver: Option<C::Receiver>,
        cmd: CMD,
        need_ack: bool,
    ) -> Result<CMD::Resp>;

    fn register_raw_handler<H: RawHandler<C>>(&mut self, name: &str, hdl: H) -> Result<()>;

    fn unregister_raw_handler<H: RawHandler<C>>(&mut self, name: &str, hdl: H) -> Result<()>;
}
