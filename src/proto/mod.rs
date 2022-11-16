use std::io::Write;

use crate::Result;

pub trait Codec: Sized {
    type Sender;
    type Receiver;
    type Ident;
    type Seq;
    type Ctx;

    fn ctx(sender: Self::Sender, receiver: Self::Receiver) -> Self::Ctx;

    fn pack_msg<M: Message<Self>>(
        ctx: Self::Ctx,
        seq: Self::Seq,
        msg: M,
        need_ack: bool,
    ) -> Result<Vec<u8>>;

    #[allow(clippy::type_complexity)]
    fn unpack_raw(data: &[u8]) -> Result<((Self::Ident, Self::Seq, Self::Ctx, &[u8]), usize)>;
}

pub trait Serialize<C: Codec> {
    fn size(&self) -> usize;

    fn ser(&self, w: impl Write) -> Result<()>;
}

pub trait Deserialize<C: Codec>: Sized {
    fn de(data: &[u8]) -> Result<Self>;
}

pub trait Message<C: Codec>: Serialize<C> {
    const IDENT: C::Ident;
}

pub trait Command<C: Codec>: Message<C> {
    const RECEIVER: Option<C::Receiver>;

    type Resp: Deserialize<C>;
}
