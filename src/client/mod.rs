use crate::{
    proto::{Codec, ProtoCommand, Raw},
    Error, Result,
};

pub mod transport;
pub mod v1;

use transport::{TransportRx, TransportRxCloser, TransportTx};

pub trait RawHandler<C: Codec> {
    // return if the handler is executed
    fn recv(&self, raw: &Raw<C>) -> Result<bool>;

    fn gc(&self) -> Result<()>;
}

pub trait Connection<C: Codec>: Sized {
    fn new(
        tx: Box<dyn TransportTx>,
        rxs: Vec<Box<dyn TransportRx>>,
        closers: Vec<Box<dyn TransportRxCloser>>,
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

    fn send_cmd_sync<CMD: ProtoCommand<C>>(
        &self,
        receiver: Option<C::Receiver>,
        cmd: CMD,
    ) -> Result<CMD::Resp>
    where
        CMD::Resp: Send + 'static,
    {
        let resp = self
            .send_cmd(receiver, cmd, true)?
            .ok_or_else(|| Error::InvalidData("cmd response required but got none".into()))?;

        Ok(resp)
    }

    fn send_cmd_async<CMD: ProtoCommand<C>>(
        &self,
        receiver: Option<C::Receiver>,
        cmd: CMD,
    ) -> Result<()>
    where
        CMD::Resp: Send + 'static,
    {
        self.send_cmd(receiver, cmd, false)?;

        Ok(())
    }

    fn register_raw_handler<H: RawHandler<C> + Send + 'static>(
        &self,
        name: &str,
        hdl: H,
    ) -> Result<()>;

    fn unregister_raw_handler(&self, name: &str) -> Result<bool>;

    fn host(&self) -> C::Sender;
}
