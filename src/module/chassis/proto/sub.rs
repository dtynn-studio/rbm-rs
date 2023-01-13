use std::io::Cursor;

use crate::{
    ensure_buf_size,
    module::common::constant::v1::Uid,
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize, ProtoSubscribe,
    },
    util::{ordered::ReadOrderedExt, unit_convertor},
    Result,
};

#[derive(Debug, PartialEq, Eq)]
pub enum PositionOriginMode {
    Current,
    SwitchOn,
}

pub struct Position {
    mode: PositionOriginMode,
    pub current: PositionPush,
    origin: Option<PositionPush>,
}

impl Position {
    pub fn new(mode: PositionOriginMode) -> Self {
        Self {
            mode,
            current: Default::default(),
            origin: None,
        }
    }
}

impl ProtoSubscribe<V1> for Position {
    const SID: u64 = Uid::Position as u64;

    type Push = PositionPush;

    fn apply_push(&mut self, mut push: Self::Push) -> Result<()> {
        if self.mode == PositionOriginMode::Current {
            let origin = self.origin.get_or_insert(push);
            push.x -= origin.x;
            push.y -= origin.y;
            push.z -= origin.z;
        }

        self.current.x = unit_convertor::CHASSIS_POS_X_SUB_CONVERTOR.proto2val(push.x)?;
        self.current.y = unit_convertor::CHASSIS_POS_Y_SUB_CONVERTOR.proto2val(push.y)?;
        self.current.z = unit_convertor::CHASSIS_POS_Z_SUB_CONVERTOR.proto2val(push.z)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PositionPush {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Deserialize<V1> for PositionPush {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let x = reader.read_le()?;
        let y = reader.read_le()?;
        let z = reader.read_le()?;
        Ok(Self { x, y, z })
    }
}

#[derive(Debug, Default)]
pub struct Attitude {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Deserialize<V1> for Attitude {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let yaw = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_YAW_CONVERTOR.proto2val::<f32>(val))?;

        let pitch = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_PITCH_CONVERTOR.proto2val::<f32>(val))?;

        let roll = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_ROLL_CONVERTOR.proto2val::<f32>(val))?;

        Ok(Self { yaw, pitch, roll })
    }
}

impl_v1_sub_self!(Attitude);

#[derive(Debug, Default)]
pub struct ChassisMode {
    pub mis_type: u8,
    pub sdk_type: u8,
}

impl_v1_sub_self!(ChassisMode);

impl Deserialize<V1> for ChassisMode {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 2);
        Ok(ChassisMode {
            mis_type: buf[0],
            sdk_type: buf[1],
        })
    }
}

#[derive(Debug, Default)]
pub struct Sbus {
    pub connect_status: u8,
    pub channels: [i16; 16],
}

impl Deserialize<V1> for Sbus {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let connect_status = reader.read_le()?;

        let mut channels = [0i16; 16];
        for i in channels.as_mut_slice() {
            *i = reader.read_le()?;
        }

        Ok(Sbus {
            connect_status,
            channels,
        })
    }
}

impl_v1_sub_self!(Sbus);

#[derive(Debug, Default)]
pub struct Velocity {
    // 上电时刻世界坐标系
    pub g_x: f32,
    pub g_y: f32,
    pub g_z: f32,
    // 当前时刻车身坐标系
    pub b_x: f32,
    pub b_y: f32,
    pub b_z: f32,
}

impl Deserialize<V1> for Velocity {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let g_x = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_SPD_X_CONVERTOR.proto2val::<f32>(val))?;
        let g_y = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_SPD_Y_CONVERTOR.proto2val::<f32>(val))?;
        let g_z = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_SPD_Z_CONVERTOR.proto2val::<f32>(val))?;

        let b_x = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_SPD_X_CONVERTOR.proto2val::<f32>(val))?;
        let b_y = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_SPD_Y_CONVERTOR.proto2val::<f32>(val))?;
        let b_z = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_SPD_Z_CONVERTOR.proto2val::<f32>(val))?;

        Ok(Velocity {
            g_x,
            g_y,
            g_z,
            b_x,
            b_y,
            b_z,
        })
    }
}

impl_v1_sub_self!(Velocity);

// 底盘电调信息
#[derive(Debug, Default)]
pub struct EscItem {
    pub speed: f32,
    pub angle: f32,
    pub timestamp: u32,
    pub state: u8,
}

pub type Esc = [EscItem; 4];

impl Deserialize<V1> for Esc {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let mut info = Esc::default();

        for item in info.as_mut_slice() {
            item.speed = reader.read_le()?;
        }

        for item in info.as_mut_slice() {
            item.angle = reader.read_le()?;
        }

        for item in info.as_mut_slice() {
            item.timestamp = reader.read_le()?;
        }

        for item in info.as_mut_slice() {
            item.state = reader.read_le()?;
        }

        Ok(info)
    }
}

impl_v1_sub_self!(Esc);

// 陀螺仪信息
#[derive(Debug, Default)]
pub struct Imu {
    // 加速度
    pub acc_x: f32,
    pub acc_y: f32,
    pub acc_z: f32,

    // 角速度
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
}

impl Deserialize<V1> for Imu {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);

        let acc_x = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_ACC_CONVERTOR.proto2val::<f32>(val))?;
        let acc_y = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_ACC_CONVERTOR.proto2val::<f32>(val))?;
        let acc_z = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_ACC_CONVERTOR.proto2val::<f32>(val))?;

        let gyro_x = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_GYRO_CONVERTOR.proto2val::<f32>(val))?;
        let gyro_y = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_GYRO_CONVERTOR.proto2val::<f32>(val))?;
        let gyro_z = reader
            .read_le()
            .and_then(|val| unit_convertor::CHASSIS_GYRO_CONVERTOR.proto2val::<f32>(val))?;

        Ok(Imu {
            acc_x,
            acc_y,
            acc_z,
            gyro_x,
            gyro_y,
            gyro_z,
        })
    }
}

impl_v1_sub_self!(Imu);

// :static_flag: 状态标准位
// :up_hill: 处于上坡状态
// :down_hill: 处于下坡状态
// :on_slope: 处于倾斜状态
// :is_pickup: 处于抱起状态
// :slip_flag: 车身打滑
// :impact_x: x轴发生撞击
// :impact_y: y轴发生撞击
// :impact_z: z轴发生撞击
// :roll_over: 车身翻转
// :hill_static: 处于斜坡状态
#[derive(Debug, Default)]
pub struct SaStatus {
    pub static_flag: bool,
    pub up_hill: bool,
    pub down_hill: bool,
    pub on_slope: bool,
    pub is_pick_up: bool,
    pub slip_flag: bool,
    pub impact_x: bool,
    pub impact_y: bool,
    pub impact_z: bool,
    pub roll_over: bool,
    pub hill_static: bool,
}

impl Deserialize<V1> for SaStatus {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 2);
        Ok(SaStatus {
            static_flag: buf[0] & 0x01 == 1,
            up_hill: (buf[0] >> 1) & 0x01 == 1,
            down_hill: (buf[0] >> 2) & 0x01 == 1,
            on_slope: (buf[0] >> 3) & 0x01 == 1,
            is_pick_up: (buf[0] >> 4) & 0x01 == 1,
            slip_flag: (buf[0] >> 5) & 0x01 == 1,
            impact_x: (buf[0] >> 6) & 0x01 == 1,
            impact_y: (buf[0] >> 7) & 0x01 == 1,
            impact_z: buf[1] & 0x01 == 1,
            roll_over: (buf[1] >> 1) & 0x01 == 1,
            hill_static: (buf[1] >> 2) & 0x01 == 1,
        })
    }
}

impl_v1_sub_self!(SaStatus);