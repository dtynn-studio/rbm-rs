use std::io::{Cursor, Write};

use crate::{
    proto::{
        v1::{
            action::ActionUpdateHead, cset::CMD_SET_CTRL, impl_v1_action_update, impl_v1_msg,
            Receiver, V1,
        },
        ActionState, Deserialize, ProtoAction, Serialize, ToProtoMessage,
    },
    util::{
        host2byte,
        ordered::{ReadOrderedExt, WriteOrderedExt},
    },
    Result,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MoveMode {
    Rel = 0,
    Abs = 1,
}

#[derive(Debug)]
pub struct Move<S: Default> {
    msg: MoveMsg,

    pub progress: MoveUpadte,
    pub status: S,
}

impl<S: Default> Move<S> {
    pub fn new(mode: MoveMode, x: i32, y: i32) -> Self {
        // ROBOTIC_ARM_POS_CHECK does nothing to the value,
        // otherwise we should check & convert x/y here
        Move {
            msg: MoveMsg::new(mode, x, y, 0, 0x03),

            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl ToProtoMessage<V1> for Move<ActionUpdateHead> {
    type Message = MoveMsg;

    fn to_proto_message(&self) -> Result<Self::Message> {
        Ok(self.msg)
    }
}

impl ProtoAction<V1> for Move<ActionUpdateHead> {
    type Update = MoveUpadte;

    const TARGET: Option<Receiver> = Some(host2byte(3, 6));

    fn apply_state(&mut self, state: ActionState) -> Result<()> {
        self.status.state = state;
        Ok(())
    }

    fn apply_update(&mut self, update: (ActionUpdateHead, Self::Update)) -> Result<bool> {
        self.progress = update.1;
        self.status = update.0;
        Ok(self.status.is_completed())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveMsg {
    id: u8,
    pub mode: MoveMode,
    pub mask: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MoveMsg {
    #[inline]
    fn new(mode: MoveMode, x: i32, y: i32, z: i32, mask: u8) -> Self {
        MoveMsg {
            id: host2byte(27, 2),
            mode,
            mask,
            x,
            y,
            z,
        }
    }
}

impl Serialize<V1> for MoveMsg {
    const SIZE_HINT: usize = 15;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.id)?;
        w.write_le(self.mode as u8)?;
        w.write_le(self.mask)?;

        w.write_le(self.x)?;
        w.write_le(self.y)?;
        w.write_le(self.z)?;

        Ok(())
    }
}

impl_v1_msg!(MoveMsg, CMD_SET_CTRL, 0xb5);

#[derive(Debug, Default)]
pub struct MoveUpadte {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Deserialize<V1> for MoveUpadte {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let x = reader.read_le()?;
        let y = reader.read_le()?;

        Ok(MoveUpadte { x, y, z: 0 })
    }
}

impl_v1_action_update!(MoveUpadte, CMD_SET_CTRL, 0xb6);
