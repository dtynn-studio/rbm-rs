use std::io::Write;

use crate::{Error, Result};

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

pub enum ProtoType {
    V1,
    Text,
}

pub trait Identified {
    type Ident;
}

pub trait Codec {
    type Msg: Identified;

    fn pack_msg(msg: Self::Msg) -> Result<Vec<u8>>;
    fn unpack_raw<'b>(
        data: &'b [u8],
    ) -> Result<(<Self::Msg as Identified>::Ident, &'b [u8], usize)>;
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
