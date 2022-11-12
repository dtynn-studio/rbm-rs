use super::V1ActionStatus;
use crate::{
    proto::{
        action::{Action, Progress},
        cmd::Command,
        host2byte,
        v1::ctrl::{GimbalActionPush, GimbalRecenter, GimbalRotate},
        Completed,
    },
    util::unit_convertor,
    Result,
};

#[derive(Debug, Default)]
pub struct GimbalMoveProgress {
    pub yaw: f32,
    pub roll: f32,
    pub pitch: f32,
}

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
    yaw: i16,   // unit: 0.1 degree
    pitch: i16, // unit: 0.1 degree

    yaw_speed: u16,
    pitch_speed: u16,

    coordinate: GimbalCoordinate,

    pub progress: GimbalMoveProgress,
    pub status: V1ActionStatus,
}

impl GimbalMoveAction {
    pub fn new(
        yaw: i16,
        pitch: i16,
        yaw_speed: u16,
        pitch_speed: u16,
        coordinate: GimbalCoordinate,
    ) -> Self {
        GimbalMoveAction {
            yaw,
            pitch,
            yaw_speed,
            pitch_speed,
            coordinate,
            progress: Default::default(),
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
            unit_convertor::GIMBAL_YAW_MOVE_SPEED_SET_CONVERTOR.val2proto(self.yaw_speed)?;

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
                self.progress.yaw = (evt.yaw as f32) / 10.0;
                self.progress.pitch = (evt.pitch as f32) / 10.0;
                self.progress.roll = (evt.roll as f32) / 10.0;
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}

pub struct GimbalRecenterAction {
    pitch_speed: u16,
    yaw_speed: u16,

    pub progress: GimbalMoveProgress,
    pub status: V1ActionStatus,
}

impl GimbalRecenterAction {
    pub fn new(pitch_speed: u16, yaw_speed: u16) -> Self {
        Self {
            pitch_speed,
            yaw_speed,

            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl Action for GimbalRecenterAction {
    type Cmd = GimbalRecenter;
    type Event = GimbalActionPush;
    type Status = V1ActionStatus;

    const RECEIVER: u8 = host2byte(4, 0);

    fn pack_cmd(&self) -> Result<Self::Cmd> {
        Ok(GimbalRecenter {
            pitch_speed: self.pitch_speed,
            yaw_speed: self.yaw_speed,
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
                self.progress.yaw = (evt.yaw as f32) / 10.0;
                self.progress.pitch = (evt.pitch as f32) / 10.0;
                self.progress.roll = (evt.roll as f32) / 10.0;
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}
