use std::io::Write;

use crate::Result;

mod action;
mod subscribe;
pub mod v1;

pub use action::*;
pub use subscribe::*;

pub struct Raw<C: Codec> {
    pub sender: C::Sender,
    pub receiver: C::Receiver,
    pub is_ack: bool,
    pub id: C::Ident,
    pub seq: C::Seq,
    pub raw_data: Vec<u8>,
}

pub trait Codec: Sized {
    type Sender;
    type Receiver;
    type Ident;
    type Seq;

    type ActionConfig;
    type ActionUpdateHead;

    type SubscribeConfig;
    type SubscribeID;

    fn pack_msg<M: ProtoMessage<Self>>(
        sender: Self::Sender,
        receiver: Self::Receiver,
        seq: Self::Seq,
        msg: &M,
        need_ack: bool,
    ) -> Result<Vec<u8>>;

    fn unpack_raw(buf: &[u8]) -> Result<(Raw<Self>, usize)>;
}

pub trait Serialize<C: Codec> {
    const SIZE_HINT: usize;

    fn ser(&self, w: &mut impl Write) -> Result<()>;

    #[inline]
    fn size(&self) -> usize {
        Self::SIZE_HINT
    }
}

pub trait Deserialize<C: Codec>: Sized {
    fn de(buf: &[u8]) -> Result<Self>;
}

pub trait ProtoMessage<C: Codec>: Serialize<C> {
    const IDENT: C::Ident;
}

pub trait ProtoPush<C: Codec>: Deserialize<C> + Send + 'static {
    const IDENT: C::Ident;
}

/// ProtoCommand: a simple request-response
pub trait ProtoCommand<C: Codec>: ProtoMessage<C> {
    type Resp: Deserialize<C> + Send + 'static;
}

pub trait ToProtoMessage<C: Codec> {
    type Message: ProtoMessage<C>;

    fn to_proto_message(&self) -> Result<Self::Message>;
}
