use std::io::{Cursor, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::{
    ensure_buf_size,
    proto::{
        v1::{
            action::ActionUpdateHead, cset::CMD_SET_CTRL, impl_v1_action_update, impl_v1_msg,
            Receiver, V1,
        },
        ActionState, Deserialize, ProtoAction, Serialize, ToProtoMessage,
    },
    util::{host2byte, unit_convertor},
    Result,
};

#[derive(Debug, Default)]
pub struct MoveProgress {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub struct Move<S: Default> {
    dest: MoveProgress,
    spd_xy: f32,
    spd_z: f32,

    pub progress: MoveProgress,
    pub status: S,
}

impl<S: Default> Move<S> {
    pub fn new(x: f32, y: f32, z: f32, spd_xy: f32, spd_z: f32) -> Self {
        Move {
            dest: MoveProgress { x, y, z },
            spd_xy,
            spd_z,
            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl ToProtoMessage<V1> for Move<ActionUpdateHead> {
    type Message = PositionMove;

    fn to_proto_message(&self) -> Result<Self::Message> {
        let pos_x = unit_convertor::CHASSIS_POS_X_SET_CONVERTOR.val2proto(self.dest.x)?;
        let pos_y = unit_convertor::CHASSIS_POS_Y_SET_CONVERTOR.val2proto(self.dest.y)?;
        let pos_z = unit_convertor::CHASSIS_POS_Z_SET_CONVERTOR.val2proto(self.dest.z)?;
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
}

impl ProtoAction<V1> for Move<ActionUpdateHead> {
    const TARGET: Option<Receiver> = Some(host2byte(3, 6));
    type Update = PositionMoveUpdate;

    fn apply_state(&mut self, state: ActionState) -> Result<()> {
        self.status.state = state;
        Ok(())
    }

    fn apply_update(&mut self, update: (ActionUpdateHead, Self::Update)) -> Result<bool> {
        self.progress.x = unit_convertor::CHASSIS_POS_X_SET_CONVERTOR.proto2val(update.1.pos_x)?;
        self.progress.y = unit_convertor::CHASSIS_POS_Y_SET_CONVERTOR.proto2val(update.1.pos_y)?;
        self.progress.z = unit_convertor::CHASSIS_POS_Z_SET_CONVERTOR.proto2val(update.1.pos_z)?;
        self.status = update.0;

        Ok(self.status.is_completed())
    }
}

impl_v1_msg!(PositionMove, CMD_SET_CTRL, 0x25);

#[derive(Debug)]
pub struct PositionMove {
    pub ctrl_mode: u8,
    pub axis_mode: u8,
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
    pub vel_xy_max: u8,
    pub agl_omg_max: i16,
}

impl Default for PositionMove {
    fn default() -> Self {
        Self {
            ctrl_mode: 0,
            axis_mode: 0,
            pos_x: 0,
            pos_y: 0,
            pos_z: 0,
            vel_xy_max: 0,
            agl_omg_max: 300,
        }
    }
}

impl Serialize<V1> for PositionMove {
    const SIZE_HINT: usize = 11;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.ctrl_mode)?;
        w.write_u8(self.axis_mode)?;

        w.write_i16::<LE>(self.pos_x)?;
        w.write_i16::<LE>(self.pos_y)?;
        w.write_i16::<LE>(self.pos_z)?;

        w.write_u8(self.vel_xy_max)?;

        w.write_i16::<LE>(self.agl_omg_max)?;

        Ok(())
    }
}

impl_v1_action_update!(PositionMoveUpdate, CMD_SET_CTRL, 0x2a);

#[derive(Debug)]
pub struct PositionMoveUpdate {
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
}

impl Deserialize<V1> for PositionMoveUpdate {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 6);
        let mut reader = Cursor::new(buf);
        let pos_x = reader.read_i16::<LE>()?;
        let pos_y = reader.read_i16::<LE>()?;
        let pos_z = reader.read_i16::<LE>()?;

        Ok(Self {
            pos_x,
            pos_y,
            pos_z,
        })
    }
}
