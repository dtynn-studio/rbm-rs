use std::io::Cursor;

use crate::{
    module::common::constant::v1::Uid,
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize,
    },
    util::ordered::ReadOrderedExt,
    Result,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ServoStateItem {
    pub valid: bool,
    pub speed: i16,
    pub angle: i16,
}

#[derive(Debug, Default)]
pub struct ServoState {
    pub recv: u8,
    pub states: [ServoStateItem; 4],
}

impl Deserialize<V1> for ServoState {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let valid: u8 = reader.read_le()?;
        let recv = reader.read_le()?;

        let mut states = [ServoStateItem::default(); 4];
        for (i, state) in states.iter_mut().enumerate() {
            state.valid = (valid >> i) & 0x01 == 1;
        }

        for state in states.as_mut_slice() {
            state.speed = reader.read_le()?;
        }

        for state in states.as_mut_slice() {
            state.angle = reader.read_le()?;
        }

        Ok(ServoState { recv, states })
    }
}

impl_v1_sub_self!(ServoState, Uid::Servo as u64);
