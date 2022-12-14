use crate::{
    proto::{
        action::{Action, Progress},
        cmd::Command,
        host2byte,
        v1::{
            ctrl::{PositionMove, PositionPush},
            V1ActionStatus,
        },
        Completed,
    },
    util::unit_convertor,
    Result,
};

#[derive(Debug, Default)]
pub struct ChassisMoveActionProgress {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub struct ChassisMoveAction {
    x: f32,
    y: f32,
    z: f32,
    spd_xy: f32,
    spd_z: f32,

    pub progress: ChassisMoveActionProgress,
    pub status: V1ActionStatus,
}

impl ChassisMoveAction {
    pub fn new(x: f32, y: f32, z: f32, spd_xy: f32, spd_z: f32) -> Self {
        ChassisMoveAction {
            x,
            y,
            z,
            spd_xy,
            spd_z,
            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl Action for ChassisMoveAction {
    type Cmd = PositionMove;
    type Event = PositionPush;
    type Status = V1ActionStatus;

    const RECEIVER: u8 = host2byte(3, 6);

    fn pack_cmd(&self) -> Result<Self::Cmd> {
        let pos_x = unit_convertor::CHASSIS_POS_X_SET_CONVERTOR.val2proto(self.x)?;
        let pos_y = unit_convertor::CHASSIS_POS_Y_SET_CONVERTOR.val2proto(self.y)?;
        let pos_z = unit_convertor::CHASSIS_POS_Z_SET_CONVERTOR.val2proto(self.z)?;
        let vel_xy_max = unit_convertor::CHASSIS_SPEED_XY_SET_CONVERTOR.val2proto(self.spd_xy)?;
        let agl_omg_max = unit_convertor::CHASSIS_SPEED_Z_SET_CONVERTOR.val2proto(self.spd_z)?;

        Ok(PositionMove {
            pos_x,
            pos_y,
            pos_z,
            vel_xy_max,
            agl_omg_max,
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
                self.progress.x =
                    unit_convertor::CHASSIS_POS_X_SET_CONVERTOR.proto2val(evt.pos_x)?;
                self.progress.y =
                    unit_convertor::CHASSIS_POS_Y_SET_CONVERTOR.proto2val(evt.pos_y)?;
                self.progress.z =
                    unit_convertor::CHASSIS_POS_Z_SET_CONVERTOR.proto2val(evt.pos_z)?;
                self.status = status;
            }
        }
        Ok(self.status.is_completed())
    }
}
