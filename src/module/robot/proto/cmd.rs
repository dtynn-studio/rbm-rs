use std::io::Write;

use byteorder::WriteBytesExt;

use crate::{
    ensure_buf_size, ensure_ok,
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, impl_v1_empty_ser, RetOK, V1},
        Deserialize, Serialize,
    },
    Error, Result,
};

impl_v1_cmd!(Mode, RetOK, CMD_SET_CTRL, 0x46);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Free = 0,
    GimbalLead = 1,
    ChassisLead = 2,
}

impl TryFrom<u8> for Mode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Mode::Free,
            1 => Mode::GimbalLead,
            2 => Mode::ChassisLead,
            other => {
                return Err(Error::InvalidData(
                    format!("unknown robot mode {}", other).into(),
                ))
            }
        })
    }
}

impl Deserialize<V1> for Mode {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 2);

        buf[1].try_into()
    }
}

impl Serialize<V1> for Mode {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(*self as u8).map_err(From::from)
    }
}

impl_v1_cmd!(GetMode, Mode, CMD_SET_CTRL, 0x47);

#[derive(Debug)]
pub struct GetMode;

impl_v1_empty_ser!(GetMode);
