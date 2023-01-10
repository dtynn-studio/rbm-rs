use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_GIMBAL, impl_v1_cmd, RetOK, V1},
        Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

impl_v1_cmd!(CtrlCode, RetOK, CMD_SET_GIMBAL, 0xd);

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum CtrlCode {
    Suspend = 0x2ab5,
    Resume = 0x7ef2,
}

impl Serialize<V1> for CtrlCode {
    const SIZE_HINT: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(*self as u16)
    }
}

impl_v1_cmd!(SetWorkMode, RetOK, CMD_SET_GIMBAL, 0x4c);

#[derive(Debug, Default)]
pub struct SetWorkMode {
    pub workmode: u8,
    pub recenter: u8,
}

impl Serialize<V1> for SetWorkMode {
    const SIZE_HINT: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.workmode, self.recenter])
            .map_err(From::from)
    }
}

impl_v1_cmd!(SetCtrlSpeed, RetOK, CMD_SET_GIMBAL, 0xc);

#[derive(Debug, Default)]
pub struct SetCtrlSpeed {
    pub yaw: i16,
    pub roll: i16,
    pub pitch: i16,
}

impl Serialize<V1> for SetCtrlSpeed {
    const SIZE_HINT: usize = 8;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.yaw)?;
        w.write_le(self.roll)?;
        w.write_le(self.pitch)?;

        // constant ctrl byte here
        w.write_le(0xdc)?;
        Ok(())
    }
}
