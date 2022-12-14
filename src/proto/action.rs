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
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Exception | Self::Rejected
        )
    }
}

pub trait ActionCommand: Command {
    type Seq;
    fn set_action_seq(&mut self, seq: Self::Seq);
}

pub trait Action {
    type Cmd: ActionCommand;
    type Event: Event + std::fmt::Debug;
    type Status: std::fmt::Debug;

    const RECEIVER: u8;

    fn pack_cmd(&self) -> Result<Self::Cmd>;

    fn is_completed(&self) -> bool;

    fn apply_progress(
        &mut self,
        progress: Progress<<Self::Cmd as Command>::Response, Self::Status, Self::Event>,
    ) -> Result<bool>;
}

#[derive(Debug)]
pub enum Progress<R: std::fmt::Debug, S: std::fmt::Debug, E: std::fmt::Debug> {
    Response(R),
    Event(S, E),
}
