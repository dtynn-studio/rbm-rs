use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_CAMERA, impl_v1_cmd, impl_v1_empty_ser, RetOK, V1},
        Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

impl_v1_cmd!(TakePhoto, RetOK, CMD_SET_CAMERA, 0x01);

#[derive(Debug, Default)]
pub struct TakePhoto;

impl Serialize<V1> for TakePhoto {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        // static take photo type 1
        w.write_all(&[1]).map_err(From::from)
    }
}

impl_v1_cmd!(SetZoom, RetOK, CMD_SET_CAMERA, 0x34);

#[derive(Debug)]
pub struct SetZoom {
    pub enable: bool,
    _zoom: f64,
    pub typ: u8,
    pub value: i16,
}

impl Default for SetZoom {
    fn default() -> Self {
        Self {
            enable: true,
            _zoom: 1.0,
            typ: 1,
            value: 1,
        }
    }
}

impl Serialize<V1> for SetZoom {
    const SIZE_HINT: usize = 5;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE_HINT];
        buf[0] = ((self.enable as u8) << 3) | self.typ;
        // this is just impossible
        // buf[4..].copy_from_slice(&self.value.to_le_bytes());
        w.write_all(&buf[..])?;
        unimplemented!("this method is obviously incorrect");
    }
}

impl_v1_cmd!(GetZoom, (), CMD_SET_CAMERA, 0x35);

#[derive(Default, Debug)]
pub struct GetZoom;

impl_v1_empty_ser!(GetZoom);

impl_v1_cmd!(SetWhiteBalance, RetOK, CMD_SET_CAMERA, 0x2c);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum WhiteBalanceType {
    Auto = 0,
    Manual = 6,
}

#[derive(Debug)]
pub struct SetWhiteBalance {
    pub typ: WhiteBalanceType,
    pub temp1: u8,
    pub temp2: u8,
    pub tint: i16,
}

impl Serialize<V1> for SetWhiteBalance {
    const SIZE_HINT: usize = 5;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE_HINT];
        buf[3..].copy_from_slice(&self.tint.to_le_bytes());
        w.write_all(&[self.typ as u8, self.temp1, self.temp2])?;
        w.write_le(self.tint)
    }
}
