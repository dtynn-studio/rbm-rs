use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use crate::{
    proto::{v1::impl_v1_cmd, RetOK, Serialize},
    Result,
};

const CMD_SET: u8 = 0x4;

impl_v1_cmd!(GimbalSetWorkMode, RetOK, 0x4c);

#[derive(Debug, Default)]
pub struct GimbalSetWorkMode {
    pub workmode: u8,
    pub recenter: u8,
}

impl Serialize for GimbalSetWorkMode {
    const SIZE: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.workmode, self.recenter])
            .map_err(From::from)
    }
}

impl_v1_cmd!(GimbalCtrl, RetOK, 0xd);

#[derive(Debug)]
pub struct GimbalCtrl {
    pub order_code: u16,
}

impl Default for GimbalCtrl {
    fn default() -> Self {
        Self { order_code: 0x2ab5 }
    }
}

impl Serialize for GimbalCtrl {
    const SIZE: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LE>(self.order_code).map_err(From::from)
    }
}
