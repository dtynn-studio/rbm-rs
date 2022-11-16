use std::io::Write;

use crate::Result;

pub mod v1;

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

    #[allow(clippy::type_complexity)]
    fn unpack_raw(
        buf: &[u8],
    ) -> Result<(
        (
            Self::Sender,
            Self::Receiver,
            bool,
            Self::Ident,
            Self::Seq,
            &[u8],
        ),
        usize,
    )>;
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
