use super::V1ActionStatus;
use crate::{
    proto::{
        action::{Action, Progress},
        cmd::Command,
        host2byte,
        v1::ctrl::{RoboticArmMoveCtrl, RoboticArmMovePush},
        Completed,
    },
    Result,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum RoboticArmMoveMode {
    Absolutely = 0,
    Relatively = 1,
}

#[derive(Debug, Default)]
pub struct RoboticArmMoveActionProgress {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct RoboticArmMoveAction {
    x: i32,
    y: i32,
    z: i32,
    mode: RoboticArmMoveMode,

    pub progress: RoboticArmMoveActionProgress,
    pub status: V1ActionStatus,
}

impl RoboticArmMoveAction {
    pub fn new(x: i32, y: i32, z: i32, mode: RoboticArmMoveMode) -> Self {
        Self {
            x,
            y,
            z,
            mode,
            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl Action for RoboticArmMoveAction {
    type Cmd = RoboticArmMoveCtrl;
    type Event = RoboticArmMovePush;
    type Status = V1ActionStatus;

    const RECEIVER: u8 = host2byte(3, 6);

    fn pack_cmd(&self) -> Result<Self::Cmd> {
        Ok(RoboticArmMoveCtrl {
            x: self.x,
            y: self.y,
            z: self.z,
            mode: self.mode as u8,
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
                self.progress.x = evt.x;
                self.progress.y = evt.y;
                self.progress.z = evt.z;
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}
