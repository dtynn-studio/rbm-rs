use std::io::Write;

use crate::{
    proto::{impl_empty_ser, v1::impl_v1_cmd, RetOK, Serialize},
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

impl_v1_cmd!(SetZoom, RetOK, 0x34);

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

impl Serialize for SetZoom {
    const SIZE: usize = 5;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE];
        let enable_bit: u8 = self.enable.into();
        buf[0] = enable_bit << 3 | self.typ;
        // this is just impossible
        // buf[4..].copy_from_slice(&self.value.to_le_bytes());
        w.write_all(&buf[..])?;
        unimplemented!("this method is obviously incorrect");
    }
}

impl_v1_cmd!(GetZoom, (), 0x35);

#[derive(Default, Debug)]
pub struct GetZoom;

impl_empty_ser!(GetZoom);

impl_v1_cmd!(SetWhiteBalance, RetOK, 0x2c);

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

impl Serialize for SetWhiteBalance {
    const SIZE: usize = 5;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE];
        buf[0] = self.typ as u8;
        buf[1] = self.temp1;
        buf[2] = self.temp2;
        buf[3..].copy_from_slice(&self.tint.to_le_bytes());
        w.write_all(&buf[..]).map_err(From::from)
    }
}
