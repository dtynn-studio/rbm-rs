use std::io::Write;

use super::DetectType;
use crate::{
    ensure_buf_size, ensure_ok,
    proto::{
        v1::{cset::CMD_SET_VISION, impl_v1_cmd, impl_v1_empty_ser, RetOK, V1},
        Deserialize, Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

impl_v1_cmd!(DetectTypeMask, RetOK, CMD_SET_VISION, 0xa3);

#[derive(Debug, Clone, Copy, Default)]
pub struct DetectTypeMask(u16);

impl DetectTypeMask {
    pub fn add(&mut self, typ: DetectType) {
        self.0 &= typ.mask();
    }

    pub fn sub(&mut self, typ: DetectType) {
        self.0 ^= typ.mask();
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn enabled(&self, typ: DetectType) -> bool {
        (self.0 & typ.mask()) != 0
    }
}

impl From<DetectTypeMask> for u16 {
    fn from(v: DetectTypeMask) -> Self {
        v.0
    }
}

impl Serialize<V1> for DetectTypeMask {
    const SIZE_HINT: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.0)
    }
}

impl Deserialize<V1> for DetectTypeMask {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 3);
        Ok(DetectTypeMask(buf[1] as u16 | (buf[2] as u16) << 8))
    }
}

impl_v1_cmd!(DetectStatus, DetectTypeMask, CMD_SET_VISION, 0xa5);

#[derive(Debug)]
pub struct DetectStatus;

impl_v1_empty_ser!(DetectStatus);

impl_v1_cmd!(SetColor, RetOK, CMD_SET_VISION, 0xab);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Line = 1,
    Marker = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red = 1,
    Green = 2,
    Blue = 3,
}

#[derive(Debug)]
pub struct SetColor {
    pub typ: ColorType,
    pub color: Color,
}

impl Serialize<V1> for SetColor {
    const SIZE_HINT: usize = 2;

    fn ser(&self, w: &mut impl std::io::Write) -> Result<()> {
        w.write_all(&[self.typ as u8, self.color as u8])
            .map_err(From::from)
    }
}
