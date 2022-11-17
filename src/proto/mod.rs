use std::io::Write;

use crate::Result;

pub mod v1;

pub type Raw<'r, C> = (
    <C as Codec>::Sender,
    <C as Codec>::Receiver,
    bool,
    <C as Codec>::Ident,
    <C as Codec>::Seq,
    &'r [u8],
);

pub trait Codec: Sized {
    type Sender;
    type Receiver;
    type Ident;
    type Seq;

    fn pack_msg<M: Message<Self>>(
        sender: Self::Sender,
        receiver: Self::Receiver,
        seq: Self::Seq,
        msg: M,
        need_ack: bool,
    ) -> Result<Vec<u8>>;

    fn unpack_raw(buf: &[u8]) -> Result<(Raw<Self>, usize)>;
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
