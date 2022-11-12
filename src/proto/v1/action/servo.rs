use super::V1ActionStatus;
use crate::{
    proto::{
        action::{Action, Progress},
        cmd::Command,
        host2byte,
        v1::ctrl::{ServoCtrlPush, ServoCtrlSet},
        Completed,
    },
    Result,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ServoIndex {
    No1 = 1,
    No2 = 2,
    No3 = 3,
}

#[derive(Debug, Default)]
pub struct ServoSetAngleActionProgress {
    pub angle: i32,
}

#[derive(Debug)]
pub struct ServoSetAngleAction {
    index: ServoIndex,
    value: i32,

    pub progress: ServoSetAngleActionProgress,
    pub status: V1ActionStatus,
}

impl ServoSetAngleAction {
    pub fn new(index: ServoIndex, value: i32) -> Self {
        Self {
            index,
            value,
            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl Action for ServoSetAngleAction {
    type Cmd = ServoCtrlSet;
    type Event = ServoCtrlPush;
    type Status = V1ActionStatus;

    const RECEIVER: u8 = host2byte(3, 6);

    fn pack_cmd(&self) -> Result<Self::Cmd> {
        Ok(ServoCtrlSet {
            id: self.index as u8,
            value: (self.value + 180) * 10,
            ..Default::default()
        })
    }

    fn is_completed(&self) -> bool {
        self.status.is_completed()
    }

    fn apply_progress(
        &mut self,
        progress: Progress<<Self::Cmd as Command>::Response, Self::Status, Self::Event>,
    ) -> Result<bool> {
        match progress {
            Progress::Response(resp) => {
                self.status.state = resp.into();
            }

            Progress::Event(status, evt) => {
                self.progress.angle = evt.value;
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}
