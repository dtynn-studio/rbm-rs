use std::hash::Hash;
use std::io::Write;

use crate::{ensure_ok, Error, Result};

mod v1;

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

pub trait Codec {
    type CmdIdent;
    type MsgIdent: Eq + Hash;
    type Ctx: CodecCtx;

    fn ctx<M: Message<Ident = Self::CmdIdent>>(
        sender: u8,
        receiver: u8,
        need_ack: Option<DussMBAck>,
    ) -> Self::Ctx;

    fn pack_msg<M: Message<Ident = Self::CmdIdent>>(
        &self,
        ctx: Self::Ctx,
        msg: M,
    ) -> Result<(Self::MsgIdent, Vec<u8>)>;

    #[allow(clippy::type_complexity)]
    fn unpack_raw(buf: &[u8]) -> Result<(Self::MsgIdent, Self::Ctx, &[u8], usize)>;
}

pub trait Message: std::fmt::Debug + Serialize {
    type Ident;
    type Response: std::fmt::Debug + Deserialize;

    const IDENT: Self::Ident;
    const CMD_TYPE: DussMBType = DussMBType::Req;
}

pub trait Serialize {
    const SIZE: usize;

    fn ser(&self, w: &mut impl Write) -> Result<()>;
}

pub trait Deserialize: Sized {
    fn de(buf: &[u8]) -> Result<Self>;
}

#[derive(Debug)]
pub struct RetOK;

impl Deserialize for RetOK {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        Ok(RetOK)
    }
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
