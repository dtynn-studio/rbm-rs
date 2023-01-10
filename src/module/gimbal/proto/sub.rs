use std::io::Cursor;

use crate::{
    module::common::constant::v1::Uid,
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize,
    },
    util::{ordered::ReadOrderedExt, unit_convertor},
    Result,
};

// :pitch_angle: 相对底盘的pitch轴角度
// :yaw_angle: 相对底盘的yaw轴角度
// :pitch_ground_angle: 上电时刻pitch轴角度
// :yaw_ground_angle: 上电时刻yaw轴角度

#[derive(Debug, Default)]
pub struct Position {
    pub yaw_angle: f32,
    pub pitch_angle: f32,
    pub yaw_ground_angle: f32,
    pub pitch_ground_angle: f32,
    pub option_mode: u8,
    pub return_center: bool,
}

impl Deserialize<V1> for Position {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let yaw_ground_angle = reader
            .read_le()
            .and_then(|val| unit_convertor::GIMBAL_ATTI_YAW_CONVERTOR.proto2val::<i16>(val))?;
        let pitch_ground_angle = reader
            .read_le()
            .and_then(|val| unit_convertor::GIMBAL_ATTI_PITCH_CONVERTOR.proto2val::<i16>(val))?;

        let yaw_angle = reader
            .read_le()
            .and_then(|val| unit_convertor::GIMBAL_ATTI_YAW_CONVERTOR.proto2val::<i16>(val))?;
        let pitch_angle = reader
            .read_le()
            .and_then(|val| unit_convertor::GIMBAL_ATTI_PITCH_CONVERTOR.proto2val::<i16>(val))?;

        let res: u8 = reader.read_le()?;

        Ok(Position {
            yaw_angle,
            pitch_angle,
            yaw_ground_angle,
            pitch_ground_angle,
            option_mode: res & 0x2,
            return_center: ((res >> 2) & 0x01) == 1,
        })
    }
}

impl_v1_sub_self!(Position, Uid::GimbalPos as u64);
