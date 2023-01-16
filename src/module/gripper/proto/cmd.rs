use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_GRIPPER, impl_v1_cmd, Receiver, RetOK, V1},
        Serialize,
    },
    util::{host2byte, ordered::WriteOrderedExt},
    Result,
};

pub const HOST_ID: Receiver = host2byte(27, 1);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ControlOp {
    Open = 1,
    Close = 2,
}

impl_v1_cmd!(Control, RetOK, CMD_SET_GRIPPER, 0x11);

#[derive(Debug)]
pub struct Control {
    pub id: u8,
    pub op: ControlOp,
    pub power: u16,
}

impl Control {
    pub fn new(op: ControlOp, power: u16) -> Self {
        Control {
            id: HOST_ID,
            op,
            power,
        }
    }
}

impl Serialize<V1> for Control {
    const SIZE_HINT: usize = 4;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.id, self.op as u8])?;
        w.write_le(self.power)?;

        Ok(())
    }
}
