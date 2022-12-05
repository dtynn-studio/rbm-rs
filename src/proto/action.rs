use super::{Codec, Deserialize, ProtoCommand};
use crate::{Error, Result};

/// Action: a command to the target with a batch of updates from the target until the action is
/// done.
pub trait ProtoAction<C: Codec> {
    type Cmd: ProtoCommand<C>;
    type Update: Deserialize<C>;

    const TARGET: Option<C::Receiver>;

    fn pack_cmd(&self) -> Result<Self::Cmd>;
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

impl TryFrom<u8> for ActionState {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => ActionState::Running,
            1 => ActionState::Succeeded,
            2 => ActionState::Failed,
            3 => ActionState::Started,
            other => {
                return Err(Error::InvalidData(
                    format!("unknown action state value {}", other).into(),
                ))
            }
        };

        Ok(res)
    }
}