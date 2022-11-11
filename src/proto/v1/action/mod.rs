use crate::{
    ensure_buf_size,
    proto::{action::State, Completed, Deserialize},
    Result, RetCode,
};

mod chassis;
mod gimbal;
mod robotic_arm;
mod servo;
mod sound;

pub use chassis::*;
pub use gimbal::*;
pub use robotic_arm::*;
pub use servo::*;
pub use sound::*;

#[derive(Debug)]
pub struct V1ActionResponse {
    pub retcode: RetCode,
    pub acception: Option<u8>,
}

impl Completed for V1ActionResponse {
    fn is_completed(&self) -> bool {
        State::from(V1ActionResponse {
            retcode: self.retcode,
            acception: self.acception,
        })
        .is_completed()
    }
}

impl From<V1ActionResponse> for State {
    fn from(v: V1ActionResponse) -> Self {
        match (v.retcode, v.acception) {
            (RetCode(0), Some(0)) => State::Started,
            (RetCode(0), Some(1)) => State::Rejected,
            (RetCode(0), Some(2)) => State::Succeeded,
            _ => State::Failed,
        }
    }
}

impl Deserialize for V1ActionResponse {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let retcode: RetCode = buf[0].into();
        let acception = if retcode.is_ok() {
            ensure_buf_size!(buf, 2);
            Some(buf[1])
        } else {
            None
        };

        Ok(Self { retcode, acception })
    }
}

pub(super) const ACTION_STATUS_SIZE: usize = 3;

#[derive(Debug)]
pub struct V1ActionStatus {
    pub percent: u8,
    pub error_reason: u8,
    pub state: State,
}

impl Default for V1ActionStatus {
    fn default() -> Self {
        Self {
            percent: 0,
            error_reason: 0,
            state: State::Idle,
        }
    }
}

impl Completed for V1ActionStatus {
    fn is_completed(&self) -> bool {
        self.percent == 100 || self.state.is_completed()
    }
}
