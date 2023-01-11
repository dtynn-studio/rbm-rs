use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, RetOK, V1},
        Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FireType {
    Water = 0,
    Infrared = 1,
}

impl_v1_cmd!(Fire, RetOK, CMD_SET_CTRL, 0x51);

#[derive(Debug)]
pub struct Fire {
    pub typ: FireType,
    pub times: u8,
}

impl Fire {
    pub fn new(typ: FireType, times: u8) -> Self {
        Fire { typ, times }
    }
}

impl Serialize<V1> for Fire {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le((self.typ as u8) << 4 | self.times)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LedEffect {
    Off = 0,
    On = 1,
}

impl_v1_cmd!(SetLed, RetOK, CMD_SET_CTRL, 0x55);

#[derive(Debug)]
pub struct SetLed {
    pub mode: u8,
    pub effect: LedEffect,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub times: u8,
    pub t1: u16,
    pub t2: u16,
}

impl SetLed {
    pub fn new(effect: LedEffect, r: u8, g: u8, b: u8, times: u8) -> Self {
        SetLed {
            mode: 7,
            effect,
            r,
            g,
            b,
            times,
            t1: 100,
            t2: 100,
        }
    }
}

impl Serialize<V1> for SetLed {
    const SIZE_HINT: usize = 9;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[
            self.mode << 4 | self.effect as u8,
            self.r,
            self.g,
            self.b,
            self.times,
        ])?;

        w.write_le(self.t1)?;
        w.write_le(self.t2)?;

        Ok(())
    }
}
