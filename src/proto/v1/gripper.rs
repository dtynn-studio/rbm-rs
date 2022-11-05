use std::io::{Cursor, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::{
    ensure_buf_size, ensure_ok,
    proto::{host2byte, v1::impl_v1_cmd, Deserialize, RetOK, Serialize},
    Result,
};

const CMD_SET: u8 = 0x33;

impl_v1_cmd!(GripperCtrl, RetOK, 0x11);

#[derive(Debug)]
pub struct GripperCtrl {
    pub id: u8,
    pub control: u8,
    pub power: u16,
}

impl Default for GripperCtrl {
    fn default() -> Self {
        Self {
            id: host2byte(27, 1),
            control: 0,
            power: 330,
        }
    }
}

impl Serialize for GripperCtrl {
    const SIZE: usize = 4;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        w.write_u8(self.control)?;
        w.write_u16::<LE>(self.power)?;

        Ok(())
    }
}

impl_v1_cmd!(RoboticArmMove, RetOK, 0x13);

#[derive(Debug)]
pub struct RoboticArmMove {
    pub id: u8,
    pub typ: u8,
    pub mask: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for RoboticArmMove {
    fn default() -> Self {
        Self {
            id: host2byte(27, 1),
            typ: 0,
            mask: 0x3,
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Serialize for RoboticArmMove {
    const SIZE: usize = 15;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        w.write_u8(self.typ)?;
        w.write_u8(self.mask)?;

        w.write_i32::<LE>(self.x)?;
        w.write_i32::<LE>(self.y)?;
        w.write_i32::<LE>(self.z)?;

        Ok(())
    }
}

impl_v1_cmd!(RoboticArmGetPostion, RoboticArmGetPostionResp, 0x14);

#[derive(Debug)]
pub struct RoboticArmGetPostionResp {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Deserialize for RoboticArmGetPostionResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 1 + 13);
        let mut reader = Cursor::new(&buf[1..]);
        let x = reader.read_i32::<LE>()?;
        let y = reader.read_i32::<LE>()?;
        let z = reader.read_i32::<LE>()?;

        Ok(Self { x, y, z })
    }
}

#[derive(Debug)]
pub struct RoboticArmGetPostion {
    pub id: u8,
}

impl Default for RoboticArmGetPostion {
    fn default() -> Self {
        Self { id: 0x5b }
    }
}

impl Serialize for RoboticArmGetPostion {
    const SIZE: usize = 1;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id).map_err(From::from)
    }
}

impl_v1_cmd!(ServoModeSet, (), 0x16);

#[derive(Debug)]
pub struct ServoModeSet {
    pub id: u8,
    pub mode: u8,
}

impl Default for ServoModeSet {
    fn default() -> Self {
        Self { id: 0x19, mode: 0 }
    }
}

impl Serialize for ServoModeSet {
    const SIZE: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.id, self.mode]).map_err(From::from)
    }
}

impl_v1_cmd!(ServoControl, (), 0x17);

#[derive(Debug)]
pub struct ServoControl {
    pub id: u8,
    pub enabled: bool,
    pub value: u16,
}

impl Default for ServoControl {
    fn default() -> Self {
        Self {
            id: 0x19,
            enabled: true,
            value: 0,
        }
    }
}

impl Serialize for ServoControl {
    const SIZE: usize = 4;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        w.write_u8(self.enabled as u8)?;
        w.write_u16::<LE>(self.value)?;

        Ok(())
    }
}

impl_v1_cmd!(ServoGetAngle, ServoGetAngleResp, 0x15);

#[derive(Debug)]
pub struct ServoGetAngleResp {
    pub angle: u32,
}

impl Deserialize for ServoGetAngleResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 5);
        Ok(Self {
            angle: (buf[1] as u32
                + ((buf[2] as u32) << 8)
                + ((buf[3] as u32) << 16)
                + ((buf[4] as u32) << 24))
                / 10,
        })
    }
}

#[derive(Debug)]
pub struct ServoGetAngle {
    pub id: u8,
}

impl Default for ServoGetAngle {
    fn default() -> Self {
        Self { id: 0x19 }
    }
}

impl Serialize for ServoGetAngle {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        Ok(())
    }
}
