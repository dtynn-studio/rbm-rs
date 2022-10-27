use std::io::Write;

use crate::{
    proto::{v1::impl_v1_cmd, RetOK, Serialize},
    Result,
};

const CMD_SET: u8 = 0x02;

impl_v1_cmd!(TakePhoto, RetOK, 0x01);

#[derive(Debug)]
pub struct TakePhoto {
    pub typ: u8,
}

impl Default for TakePhoto {
    fn default() -> Self {
        Self { typ: 1 }
    }
}

impl Serialize for TakePhoto {
    const SIZE: usize = 1;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.typ]).map_err(From::from)
    }
}
