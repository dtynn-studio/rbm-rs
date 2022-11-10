use super::{ensure_buf_size, Deserialize, Serialize};
use crate::{Error, Result, RetCode};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Acception {
    Started,
    Rejected,
    Succeeded,
    Failed,
}

impl TryFrom<u8> for Acception {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => Acception::Started,
            1 => Acception::Rejected,
            2 => Acception::Succeeded,
            other => {
                return Err(Error::InvalidData(
                    format!("unknown action acception value {}", other).into(),
                ))
            }
        };

        Ok(res)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
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

impl TryFrom<u8> for State {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => State::Running,
            1 => State::Succeeded,
            2 => State::Failed,
            3 => State::Started,
            other => {
                return Err(Error::InvalidData(
                    format!("unknown action state value {}", other).into(),
                ))
            }
        };

        Ok(res)
    }
}

impl From<Acception> for State {
    fn from(val: Acception) -> Self {
        match val {
            Acception::Started => State::Started,
            Acception::Rejected => State::Rejected,
            Acception::Succeeded => State::Succeeded,
            Acception::Failed => State::Failed,
        }
    }
}

impl State {
    pub fn is_running(&self) -> bool {
        *self == State::Started || *self == State::Running
    }
}

#[derive(Debug)]
pub struct Accepted {
    pub retcode: RetCode,
    pub acception: Acception,
}

impl Deserialize for Accepted {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let retcode: RetCode = buf[0].into();
        let acception = if retcode.is_ok() {
            ensure_buf_size!(buf, 2);
            Acception::try_from(buf[1])?
        } else {
            Acception::Failed
        };

        Ok(Self { retcode, acception })
    }
}

pub trait ActionRequest: std::fmt::Debug + Serialize {}

pub trait Action {}
