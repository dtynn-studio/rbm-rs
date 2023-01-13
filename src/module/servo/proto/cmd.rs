use std::io::Write;

use super::ServoIndex;
use crate::{
    ensure_buf_size, ensure_ok,
    proto::{
        v1::{cset::CMD_SET_GRIPPER, impl_v1_cmd, V1},
        Deserialize, Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

impl_v1_cmd!(SetMode, (), CMD_SET_GRIPPER, 0x16);

#[derive(Debug)]
pub struct SetMode {
    pub idx: ServoIndex,
    pub mode: u8,
}

impl Serialize<V1> for SetMode {
    const SIZE_HINT: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.idx.into_idx(), self.mode])
            .map_err(From::from)
    }
}

impl_v1_cmd!(Control, (), CMD_SET_GRIPPER, 0x17);

#[derive(Debug)]
pub struct Control {
    pub idx: ServoIndex,
    pub enabled: bool,
    pub value: u16,
}

impl Control {
    #[inline]
    pub fn new(idx: ServoIndex, value: u16) -> Self {
        Control {
            idx,
            enabled: true,
            value,
        }
    }

    #[inline]
    pub fn disabled(idx: ServoIndex) -> Self {
        Control {
            idx,
            enabled: false,
            value: 0,
        }
    }
}

impl Serialize<V1> for Control {
    const SIZE_HINT: usize = 4;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.idx.into_idx(), self.enabled as u8])?;
        w.write_le(self.value)?;

        Ok(())
    }
}

impl_v1_cmd!(GetAngle, GetAngleResp, CMD_SET_GRIPPER, 0x15);

#[derive(Debug)]
pub struct GetAngleResp {
    pub angle: i32,
}

impl Deserialize<V1> for GetAngleResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 5);
        Ok(Self {
            angle: (buf[1] as u32
                | ((buf[2] as u32) << 8)
                | ((buf[3] as u32) << 16)
                | ((buf[4] as u32) << 24)) as i32
                / 10,
        })
    }
}

#[derive(Debug)]
pub struct GetAngle(ServoIndex);

impl From<ServoIndex> for GetAngle {
    fn from(idx: ServoIndex) -> Self {
        GetAngle(idx)
    }
}

impl Serialize<V1> for GetAngle {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.0.into_idx())?;
        Ok(())
    }
}
