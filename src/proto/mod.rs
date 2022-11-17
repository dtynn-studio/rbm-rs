use std::io::Write;

use crate::Result;

pub mod v1;

pub struct Raw<'r, C: Codec> {
    pub sender: C::Sender,
    pub receiver: C::Receiver,
    pub is_ack: bool,
    pub id: C::Ident,
    pub seq: C::Seq,
    pub raw_data: &'r [u8],
}

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

/// Command: a simple request-response
pub trait Command<C: Codec>: Message<C> {
    const RECEIVER: Option<C::Receiver>;

    type Resp: Deserialize<C>;
}

/// Action: a command to the target with a batch of updates from the target until the action is
/// done.
pub trait Action<C: Codec> {
    type Cmd: Command<C>;
    type Update: Deserialize<C>;
}

/// Subscribe: ask for the events published by the target, the incoming stream of events will not
/// be terminated until user unsub the events.
pub trait Subscribe<C: Codec> {
    type Cmd: Command<C>;
    type Event: Deserialize<C>;
}
