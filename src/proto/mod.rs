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

pub enum CodecType {
    V1,
    Text,
}

pub trait Msg {
    type Ident;
    type Ctx;

    fn ident(&self) -> Self::Ident;
    fn ctx(&self) -> Self::Ctx;
}

pub trait Codec<M: Msg> {
    fn pack_msg(msg: M) -> Result<Vec<u8>>;
    #[allow(clippy::type_complexity)]
    fn unpack_raw(buf: &[u8]) -> Result<(<M as Msg>::Ident, <M as Msg>::Ctx, &[u8], usize)>;
}

pub trait Command: std::fmt::Debug + Serialize {
    type Response: std::fmt::Debug + Deserialize;
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
