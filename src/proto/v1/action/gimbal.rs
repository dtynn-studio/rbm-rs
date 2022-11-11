use super::V1ActionStatus;
use crate::{
    proto::{
        action::{Action, Progress},
        cmd::Command,
        host2byte,
        v1::ctrl::{GimbalActionPush, GimbalRotate},
        Completed,
    },
    util::unit_convertor,
    Result,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GimbalCoordinate {
    NED = 0,
    CUR = 1,
    CAR = 2,
    PNED = 3, // pitch NED mode
    YCPN = 4, // yaw CAR, pitch NED mode
    YCPO = 5, // yaw CAR, pitch OFFSET mode
}

#[derive(Debug)]
pub struct GimbalMoveAction {
    pub yaw: i16,   // unit: 0.1 degree
    pub roll: i16,  // unit: 0.1 degree
    pub pitch: i16, // unit: 0.1 degree

    pub pitch_speed: u16,
    pub yaw_speed: u16,

    pub coordinate: GimbalCoordinate,

    pub status: V1ActionStatus,
}

impl GimbalMoveAction {
    pub fn new(
        yaw: i16,
        pitch: i16,
        pitch_speed: u16,
        yaw_speed: u16,
        coordinate: GimbalCoordinate,
    ) -> Self {
        GimbalMoveAction {
            yaw,
            roll: 0,
            pitch,
            pitch_speed,
            yaw_speed,
            coordinate,
            status: Default::default(),
        }
    }
}

impl Action for GimbalMoveAction {
    type Cmd = GimbalRotate;
    type Event = GimbalActionPush;
    type Status = V1ActionStatus;

    const RECEIVER: u8 = host2byte(4, 0);

    fn pack_cmd(&self) -> Result<Self::Cmd> {
        let pitch_speed =
            unit_convertor::GIMBAL_PITCH_MOVE_SPEED_SET_CONVERTOR.val2proto(self.pitch_speed)?;
        let yaw_speed =
            unit_convertor::GIMBAL_YAW_MOVE_SPEED_SET_CONVERTOR.val2proto(self.pitch_speed)?;

        Ok(GimbalRotate {
            pitch: self.pitch,
            yaw: self.yaw,
            pitch_speed,
            yaw_speed,
            coordinate: self.coordinate as u8,
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
                self.yaw = evt.yaw;
                self.pitch = evt.pitch;
                self.roll = evt.roll;
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}
