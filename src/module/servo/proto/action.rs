use std::io::{Cursor, Write};

use super::ServoIndex;
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

#[derive(Debug)]
pub struct SetAngle<S: Default> {
    msg: SetAngleMsg,

    pub status: S,
    pub angle: i32,
}

impl<S: Default> SetAngle<S> {
    pub fn new(idx: ServoIndex, value: i32) -> Self {
        SetAngle {
            msg: SetAngleMsg {
                idx,
                value: (value + 180) * 10,
            },

            status: Default::default(),
            angle: 0,
        }
    }
}

impl ToProtoMessage<V1> for SetAngle<ActionUpdateHead> {
    type Message = SetAngleMsg;

    fn to_proto_message(&self) -> Result<Self::Message> {
        Ok(self.msg)
    }
}

impl ProtoAction<V1> for SetAngle<ActionUpdateHead> {
    type Update = SetAngleUpdate;

    const TARGET: Option<Receiver> = Some(host2byte(3, 6));

    fn apply_state(&mut self, state: ActionState) -> Result<()> {
        self.status.state = state;
        Ok(())
    }

    fn apply_update(&mut self, update: (ActionUpdateHead, Self::Update)) -> Result<bool> {
        self.status = update.0;
        self.angle = update.1 .0;
        Ok(self.status.is_completed())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SetAngleMsg {
    idx: ServoIndex,
    value: i32,
}

impl Serialize<V1> for SetAngleMsg {
    const SIZE_HINT: usize = 5;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(host2byte(25, self.idx as u8))?;
        w.write_le(self.value)
    }
}

impl_v1_msg!(SetAngleMsg, CMD_SET_CTRL, 0xb7);

#[derive(Debug)]
pub struct SetAngleUpdate(pub i32);

impl Deserialize<V1> for SetAngleUpdate {
    fn de(buf: &[u8]) -> Result<Self> {
        let angle = Cursor::new(buf).read_le()?;
        Ok(SetAngleUpdate(angle))
    }
}

impl_v1_action_update!(SetAngleUpdate, CMD_SET_CTRL, 0xb8);
