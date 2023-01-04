use super::{Codec, ProtoPush, ToProtoMessage};
use crate::{Error, Result};

/// Action: a command to the target with a batch of updates from the target until the action is
/// done.
pub trait ProtoAction<C: Codec>: ToProtoMessage<C> {
    type Update: ProtoPush<C> + Send + 'static;

    const TARGET: Option<C::Receiver>;

    fn apply_state(&mut self, state: ActionState) -> Result<()>;

    fn apply_update(&mut self, update: (C::ActionUpdateHead, Self::Update)) -> Result<bool>;
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

impl ActionState {
    pub fn is_running(&self) -> bool {
        *self == Self::Started || *self == Self::Running
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Exception | Self::Rejected
        )
    }
}
