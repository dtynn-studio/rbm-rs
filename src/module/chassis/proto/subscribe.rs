use std::io::Cursor;

use byteorder::{ReadBytesExt, LE};

use crate::{
    module::common::constant::v1::Uid,
    proto::{
        v1::{subscribe::PushPeriodSubject, V1},
        Deserialize, ProtoSubscribe,
    },
    util::unit_convertor,
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
    type Push = PositionPush;

    fn apply_push(&mut self, mut push: Self::Push) -> Result<()> {
        if self.mode == PositionOriginMode::Current {
            let origin = self.origin.get_or_insert(push);
            push.x -= origin.x;
            push.y -= origin.y;
            push.z -= origin.z;
        }

        self.current.x = unit_convertor::CHASSIS_POS_X_SUB_CONVERTOR.proto2val(push.x)?;
        self.current.y = unit_convertor::CHASSIS_POS_X_SUB_CONVERTOR.proto2val(push.y)?;
        self.current.z = unit_convertor::CHASSIS_POS_X_SUB_CONVERTOR.proto2val(push.z)?;

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
        let x = reader.read_f32::<LE>()?;
        let y = reader.read_f32::<LE>()?;
        let z = reader.read_f32::<LE>()?;
        Ok(Self { x, y, z })
    }
}

impl PushPeriodSubject for PositionPush {
    const UID: u64 = Uid::Position as u64;
}
