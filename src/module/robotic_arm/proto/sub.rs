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

#[derive(Debug, Default)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Deserialize<V1> for Position {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let x = reader.read_le()?;
        let y = reader.read_le()?;
        Ok(Position { x, y })
    }
}

impl_v1_sub_self!(Position, Uid::Arm as u64);
