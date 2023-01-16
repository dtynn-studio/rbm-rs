use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, RetOK, V1},
        Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

impl_v1_cmd!(SetParam, RetOK, CMD_SET_CTRL, 0xc0);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum BaudRate {
    Rate9600 = 0,
    Rate19200 = 1,
    Rate38400 = 2,
    Rate57600 = 3,
    Rate115200 = 4,
}

impl Default for BaudRate {
    fn default() -> Self {
        Self::Rate9600
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum DataBit {
    Bit7 = 0,
    Bit8 = 1,
    Bit9 = 2,
    Bit10 = 3,
}

impl Default for DataBit {
    fn default() -> Self {
        Self::Bit8
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OddEven {
    None = 0,
    Odd = 1,
    Even = 2,
}

impl Default for OddEven {
    fn default() -> Self {
        Self::None
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum StopBit {
    One = 1,
    Two = 2,
}

impl Default for StopBit {
    fn default() -> Self {
        Self::One
    }
}

#[derive(Debug)]
pub struct SetParam {
    pub baud_rate: BaudRate,
    pub data_bit: DataBit,
    pub odd_even: OddEven,
    pub stop_bit: StopBit,
    pub tx_enabled: bool,
    pub rx_enabled: bool,
    pub rx_size: u16,
    pub tx_size: u16,
}

impl Serialize<V1> for SetParam {
    const SIZE_HINT: usize = 6;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(
            (self.stop_bit as u8 & 0x1) << 7
                | (self.odd_even as u8 & 0x3) << 5
                | (self.data_bit as u8 & 0x3) << 3
                | (self.baud_rate as u8 & 0x7),
        )?;

        // should be tx_enabled & rx_enabled?
        w.write_le(0xffu8)?;

        w.write_le(self.rx_size)?;
        w.write_le(self.tx_size)?;
        Ok(())
    }
}

impl_v1_cmd!(MsgSend, RetOK, CMD_SET_CTRL, 0xc1);

#[derive(Debug)]
pub struct MsgSend {
    typ: u8,
    pub msg: Vec<u8>,
}

impl MsgSend {
    pub fn new(msg: Vec<u8>) -> Self {
        Self { typ: 0x2, msg }
    }
}

impl Serialize<V1> for MsgSend {
    const SIZE_HINT: usize = 3;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.typ)?;
        w.write_le(self.msg.len() as u16)?;
        w.write_all(&self.msg)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        Self::SIZE_HINT + self.msg.len()
    }
}
