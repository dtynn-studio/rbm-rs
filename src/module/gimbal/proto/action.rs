use std::io::{Cursor, Write};

use crate::{
    proto::{
        v1::{
            action::ActionUpdateHead, cset::CMD_SET_CTRL, impl_v1_action_update, impl_v1_msg,
            Receiver, V1,
        },
        ActionState, Deserialize, ProtoAction, Serialize, ToProtoMessage,
    },
    util::{
        host2byte,
        ordered::{ReadOrderedExt, WriteOrderedExt},
        unit_convertor,
    },
    Result,
};

const DEFAULT_TARGET: Option<Receiver> = Some(host2byte(4, 0));

#[derive(Debug, Default)]
pub struct MoveProgress {
    pub yaw: f32,
    pub roll: f32,
    pub pitch: f32,
}

impl_v1_action_update!(MoveUpdate, CMD_SET_CTRL, 0xb1);

#[derive(Debug)]
pub struct MoveUpdate {
    pub yaw: i16,
    pub roll: i16,
    pub pitch: i16,
}

impl From<MoveUpdate> for MoveProgress {
    fn from(val: MoveUpdate) -> Self {
        MoveProgress {
            yaw: val.yaw as f32 / 10.0,
            roll: val.roll as f32 / 10.0,
            pitch: val.pitch as f32 / 10.0,
        }
    }
}

impl Deserialize<V1> for MoveUpdate {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);

        let yaw = reader.read_le()?;
        let roll = reader.read_le()?;
        let pitch = reader.read_le()?;

        Ok(MoveUpdate { yaw, roll, pitch })
    }
}

#[derive(Debug)]
pub struct Recenter<S: Default> {
    yaw_speed: f32,
    pitch_speed: f32,

    pub progress: MoveProgress,
    pub status: S,
}

impl<S: Default> Recenter<S> {
    pub fn new(yaw_speed: f32, pitch_speed: f32) -> Self {
        Recenter {
            yaw_speed,
            pitch_speed,
            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl ToProtoMessage<V1> for Recenter<ActionUpdateHead> {
    type Message = RecenterMsg;

    fn to_proto_message(&self) -> Result<Self::Message> {
        let msg = RecenterMsg {
            yaw_speed: unit_convertor::GIMBAL_RECENTER_SPEED_CONVERTOR
                .val2proto(self.yaw_speed)
                .map(Some)?,
            roll_speed: None,
            pitch_speed: unit_convertor::GIMBAL_RECENTER_SPEED_CONVERTOR
                .val2proto(self.pitch_speed)
                .map(Some)?,
        };

        Ok(msg)
    }
}

impl ProtoAction<V1> for Recenter<ActionUpdateHead> {
    const TARGET: Option<Receiver> = DEFAULT_TARGET;
    type Update = MoveUpdate;

    fn apply_state(&mut self, state: ActionState) -> Result<()> {
        self.status.state = state;
        Ok(())
    }

    fn apply_update(&mut self, update: (ActionUpdateHead, Self::Update)) -> Result<bool> {
        self.progress = update.1.into();
        self.status = update.0;
        Ok(self.status.is_completed())
    }
}

impl_v1_msg!(RecenterMsg, CMD_SET_CTRL, 0xb2);

#[derive(Debug)]
pub struct RecenterMsg {
    yaw_speed: Option<u16>,
    roll_speed: Option<u16>,
    pitch_speed: Option<u16>,
}

impl Serialize<V1> for RecenterMsg {
    const SIZE_HINT: usize = 7;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let valid_mask = (self.yaw_speed.is_some() as u8 & 0x01)
            | (self.roll_speed.is_some() as u8 & 0x01) << 1
            | (self.pitch_speed.is_some() as u8 & 0x01) << 2;

        w.write_le(valid_mask)?;
        w.write_le(self.yaw_speed.unwrap_or(0))?;
        w.write_le(self.roll_speed.unwrap_or(0))?;
        w.write_le(self.pitch_speed.unwrap_or(0))?;
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Coordinate {
    NED = 0,
    CUR = 1,
    CAR = 2,
    PNED = 3, // pitch NED mode
    YCPN = 4, // yaw CAR, pitch NED mode
    YCPO = 5, // yaw CAR, pitch OFFSET mode
}

#[derive(Debug)]
pub struct Move<S: Default> {
    coordinate: Coordinate,
    dest: (i16, i16),
    speed: (u16, u16),

    pub progress: MoveProgress,
    pub status: S,
}

impl<S: Default> Move<S> {
    pub fn new_move(dest: (f32, f32), speed: (f32, f32)) -> Result<Self> {
        let m = Move {
            coordinate: Coordinate::CUR,
            dest: (
                unit_convertor::GIMBAL_YAW_MOVE_CONVERTOR.val2proto(dest.0)?,
                unit_convertor::GIMBAL_PITCH_MOVE_CONVERTOR.val2proto(dest.1)?,
            ),
            speed: (
                unit_convertor::GIMBAL_YAW_MOVE_SPEED_SET_CONVERTOR.val2proto(speed.0)?,
                unit_convertor::GIMBAL_PITCH_MOVE_SPEED_SET_CONVERTOR.val2proto(speed.1)?,
            ),

            progress: Default::default(),
            status: Default::default(),
        };

        Ok(m)
    }

    pub fn new_move_to(dest: (f32, f32), speed: (f32, f32)) -> Result<Self> {
        let m = Move {
            coordinate: Coordinate::YCPN,
            dest: (
                unit_convertor::GIMBAL_YAW_TARGET_CONVERTOR.val2proto(dest.0)?,
                unit_convertor::GIMBAL_PITCH_TARGET_CONVERTOR.val2proto(dest.1)?,
            ),
            speed: (
                unit_convertor::GIMBAL_YAW_MOVE_SPEED_SET_CONVERTOR.val2proto(speed.0)?,
                unit_convertor::GIMBAL_PITCH_MOVE_SPEED_SET_CONVERTOR.val2proto(speed.1)?,
            ),

            progress: Default::default(),
            status: Default::default(),
        };

        Ok(m)
    }
}

impl ToProtoMessage<V1> for Move<ActionUpdateHead> {
    type Message = RotateMsg;

    fn to_proto_message(&self) -> Result<Self::Message> {
        Ok(RotateMsg {
            coordinate: self.coordinate,
            yaw: Some(self.dest.0),
            roll: None,
            pitch: Some(self.dest.1),
            yaw_speed: self.speed.0,
            roll_speed: 0,
            pitch_speed: self.speed.1,
        })
    }
}

impl ProtoAction<V1> for Move<ActionUpdateHead> {
    const TARGET: Option<Receiver> = DEFAULT_TARGET;
    type Update = MoveUpdate;

    fn apply_state(&mut self, state: ActionState) -> Result<()> {
        self.status.state = state;
        Ok(())
    }

    fn apply_update(&mut self, update: (ActionUpdateHead, Self::Update)) -> Result<bool> {
        self.progress = update.1.into();
        self.status = update.0;
        Ok(self.status.is_completed())
    }
}

impl_v1_msg!(RotateMsg, CMD_SET_CTRL, 0xb0);

#[derive(Debug)]
pub struct RotateMsg {
    coordinate: Coordinate,
    yaw: Option<i16>,
    roll: Option<i16>,
    pitch: Option<i16>,
    yaw_speed: u16,
    roll_speed: u16,
    pitch_speed: u16,
}

impl Serialize<V1> for RotateMsg {
    const SIZE_HINT: usize = 15;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let valid_mask = (self.yaw.is_some() as u8 & 0x01)
            | (self.roll.is_some() as u8 & 0x01) << 1
            | (self.pitch.is_some() as u8 & 0x01) << 2;

        w.write_le(valid_mask | ((self.coordinate as u8) << 3))?;
        w.write_le(self.yaw.unwrap_or(0))?;
        w.write_le(self.roll.unwrap_or(0))?;
        w.write_le(self.pitch.unwrap_or(0))?;

        // unused field: _error
        w.write_le(0u16)?;

        w.write_le(self.yaw_speed)?;
        w.write_le(self.roll_speed)?;
        w.write_le(self.pitch_speed)?;

        Ok(())
    }
}
