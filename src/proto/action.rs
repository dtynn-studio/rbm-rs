use super::{cmd::Command, Event};
use crate::{Error, Result};

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

impl State {
    pub fn is_running(&self) -> bool {
        *self == State::Started || *self == State::Running
    }

    pub fn is_completed(&self) -> bool {
        match self {
            Self::Succeeded | Self::Failed | Self::Exception | Self::Rejected => true,
            _ => false,
        }
    }
}

pub trait Action {
    type Cmd: Command;
    type Event: Event;
    type Status;

    fn pack_cmd(&self) -> Result<Self::Cmd>;

    fn apply_cmd_resp(&mut self, resp: <Self::Cmd as Command>::Response) -> Result<bool>;

    fn apply_event(&mut self, status: Self::Status, evt: Self::Event) -> Result<bool>;
}
