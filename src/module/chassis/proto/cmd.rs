use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use crate::{
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, RetOK, V1},
        Serialize,
    },
    Result,
};

// impl_v1_cmd!(SetWorkMode, RetOK, CMD_SET_CTRL, 0x19);

// #[derive(Debug, Default)]
// pub struct SetWorkMode {
//     pub mode: u8,
// }

// impl Serialize<V1> for SetWorkMode {
//     const SIZE_HINT: usize = 1;

//     fn ser(&self, w: &mut impl Write) -> Result<()> {
//         w.write_u8(self.mode).map_err(From::from)
//     }
// }

impl_v1_cmd!(StickOverlayMode, RetOK, CMD_SET_CTRL, 0x28);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum StickOverlayMode {
    Disabled = 0,
    ChassisMode = 1,
    GimbalMode = 2,
}

impl Serialize<V1> for StickOverlayMode {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[*self as u8]).map_err(From::from)
    }
}

// 底盘轮子速度控制
impl_v1_cmd!(SetWheelSpeed, RetOK, CMD_SET_CTRL, 0x20);

#[derive(Debug, Default)]
pub struct SetWheelSpeed {
    pub w1_spd: i16,
    pub w2_spd: i16,
    pub w3_spd: i16,
    pub w4_spd: i16,
}

impl Serialize<V1> for SetWheelSpeed {
    const SIZE_HINT: usize = 8;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_i16::<LE>(self.w1_spd)?;
        w.write_i16::<LE>(self.w2_spd)?;
        w.write_i16::<LE>(self.w3_spd)?;
        w.write_i16::<LE>(self.w4_spd)?;
        Ok(())
    }
}

// 底盘运动速度控制
impl_v1_cmd!(SetSpeed, RetOK, CMD_SET_CTRL, 0x21);

#[derive(Debug, Default)]
pub struct SetSpeed {
    pub x_spd: f32,
    pub y_spd: f32,
    pub z_spd: f32,
}

impl Serialize<V1> for SetSpeed {
    const SIZE_HINT: usize = 12;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_f32::<LE>(self.x_spd)?;
        w.write_f32::<LE>(self.y_spd)?;
        w.write_f32::<LE>(self.z_spd)?;
        Ok(())
    }
}

// PWM 输出占空比控制
impl_v1_cmd!(SetPwmPercent, RetOK, CMD_SET_CTRL, 0x3c);

#[derive(Debug, Default)]
pub struct SetPwmPercent {
    pub mask: u8,
    pub pwms: [u16; 6],
}

impl Serialize<V1> for SetPwmPercent {
    const SIZE_HINT: usize = 13;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.mask)?;
        for v in self.pwms {
            w.write_u16::<LE>(v)?;
        }
        Ok(())
    }
}

// PWM 输出频率控制
impl_v1_cmd!(SetPwmFreq, RetOK, CMD_SET_CTRL, 0x2b);

#[derive(Debug, Default)]
pub struct SetPwmFreq {
    pub mask: u8,
    pub pwms: [u16; 6],
}

impl Serialize<V1> for SetPwmFreq {
    const SIZE_HINT: usize = 13;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.mask)?;
        for v in self.pwms {
            w.write_u16::<LE>(v)?;
        }
        Ok(())
    }
}
