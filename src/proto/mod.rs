use std::io::Write;
use std::{borrow::Cow, hash::Hash};

use crate::{ensure_ok, Error, Result};

mod util;
pub mod v1;

pub use util::{byte2host, host2byte};

pub const RM_SDK_FIRST_SEQ_ID: u16 = 10000;
pub const RM_SDK_LAST_SEQ_ID: u16 = 20000;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DussMBAck {
    No = 0,
    Now = 1,
    Finish = 2,
}

impl TryFrom<u8> for DussMBAck {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::No),
            1 => Ok(Self::Now),
            2 => Ok(Self::Finish),
            _other => Err(Error::InvalidData("invalid DussMBAck".into())),
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DussMBEncrypt {
    No = 0,
    Aes128 = 1,
    Custom = 2,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DussMBType {
    Req = 0,
    Push = 1,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CodecType {
    V1,
    Text,
}

pub enum CodecIdent {
    V1(u8, u8),
    Text,
}

pub trait CodecCtx {
    fn need_ack(&self) -> DussMBAck;
    fn is_ask(&self) -> bool;
}

#[allow(clippy::type_complexity)]
pub trait Codec: Default + Send + Sync {
    type CmdIdent: Eq + Hash + Send + std::fmt::Debug;
    type Seq: Eq + Hash + Send + std::fmt::Debug;
    type Ctx: CodecCtx + Send + std::fmt::Debug;

    fn ctx<M: Message<Ident = Self::CmdIdent>>(
        sender: u8,
        receiver: u8,
        need_ack: Option<DussMBAck>,
    ) -> Self::Ctx;

    fn pack_msg<M: Message<Ident = Self::CmdIdent>>(
        &self,
        ctx: Self::Ctx,
        msg: M,
    ) -> Result<((Self::CmdIdent, Self::Seq), Vec<u8>)>;

    #[allow(clippy::type_complexity)]
    fn unpack_raw(buf: &[u8]) -> Result<((Self::CmdIdent, Self::Seq), Self::Ctx, &[u8], usize)>;
}

pub trait Message: std::fmt::Debug + Serialize {
    type Ident;
    type Response: std::fmt::Debug + Deserialize;

    const IDENT: Self::Ident;
    const CMD_TYPE: DussMBType = DussMBType::Req;
}

pub trait Event: std::fmt::Debug + Deserialize {
    type Ident;

    const IDENT: Self::Ident;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ActionState {
    Idle,
    Running,
    Succeeded,
    Failed,
    Started,
    Aborting,
    Aborted,
    Rejected,
    Exception,
}

impl ActionState {
    pub fn is_running(&self) -> bool {
        *self == ActionState::Started || *self == ActionState::Running
    }
}

pub trait Action {
    type Message: Message;
    type Event: Event;

    fn update(&mut self, event: Self::Event) -> Result<()>;

    fn state(&self) -> ActionState;

    fn percent(&self) -> f64;

    fn failure_reason(&self) -> Option<Cow<'static, str>>;

    fn apply_state(&mut self, state: ActionState) -> Result<()>;

    fn apply_event(&mut self, event: Self::Event) -> Result<()>;
}

pub trait Serialize {
    const SIZE: usize;

    fn ser(&self, w: &mut impl Write) -> Result<()>;
}

pub trait Deserialize: Sized {
    fn de(buf: &[u8]) -> Result<Self>;
}

macro_rules! impl_empty_ser {
    ($name:ty) => {
        impl $crate::proto::Serialize for $name {
            const SIZE: usize = 0;

            fn ser(&self, _w: &mut impl std::io::Write) -> $crate::Result<()> {
                Ok(())
            }
        }
    };
}

macro_rules! impl_empty_de {
    ($name:ty) => {
        impl $crate::proto::Deserialize for $name {
            fn de(_buf: &[u8]) -> $crate::Result<Self> {
                Ok(Self::default())
            }
        }
    };
}

pub(self) use impl_empty_de;
pub(self) use impl_empty_ser;

#[derive(Debug)]
pub struct RetOK;

impl Deserialize for RetOK {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        Ok(RetOK)
    }
}

impl_empty_de!(());
