use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, RetOK, V1},
        Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

impl_v1_cmd!(SetSystemLed, RetOK, CMD_SET_CTRL, 0x33);

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Comp {
    BOTTOM_BACK = 0x1,
    BOTTOM_FRONT = 0x2,
    BOTTOM_LEFT = 0x4,
    BOTTOM_RIGHT = 0x8,
    TOP_LEFT = 0x10,
    TOP_RIGHT = 0x20,
    TOP_ALL = 0x30,
    BOTTOM_ALL = 0xf,
    ALL = 0x3f,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Effect {
    Off = 0,
    On = 1,
    Breath = 2,
    Flash = 3,
    Scrolling = 4,
}

#[derive(Debug)]
pub struct SetSystemLed {
    pub comp_mask: Comp,
    pub led_mask: i16,
    pub ctrl_mode: u8,
    pub effect: Effect,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    loop_: u8,
    pub t1: i16,
    pub t2: i16,
}

impl SetSystemLed {
    pub fn new(comp: Comp, ctrl_mode: u8, effect: Effect, r: u8, g: u8, b: u8) -> Self {
        SetSystemLed {
            comp_mask: comp,
            led_mask: 0xff,
            ctrl_mode,
            effect,
            r,
            g,
            b,
            loop_: 0,
            t1: 100,
            t2: 100,
        }
    }
}

impl Serialize<V1> for SetSystemLed {
    const SIZE_HINT: usize = 15;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.comp_mask as u32)?;
        w.write_le(self.led_mask)?;

        w.write_all(&[
            self.ctrl_mode << 4 | self.effect as u8,
            self.r,
            self.g,
            self.b,
            self.loop_,
        ])?;

        w.write_le(self.t1)?;
        w.write_le(self.t2)?;

        Ok(())
    }
}
