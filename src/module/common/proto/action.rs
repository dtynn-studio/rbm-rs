use std::io::Write;

use crate::{
    ensure_buf_size,
    module::common::constant::v1::{Sound, DEFAULT_TARGET},
    proto::{
        v1::{
            action::ActionUpdateHead, cset::CMD_SET_CTRL, impl_v1_action_update, impl_v1_msg,
            Receiver, V1,
        },
        ActionState, Deserialize, ProtoAction, Serialize, ToProtoMessage,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

#[derive(Debug)]
pub struct PlaySound<S: Default> {
    to_play: PlaySoundMsg,

    pub progress: PlaySoundUpdate,
    pub status: S,
}

impl<S: Default> PlaySound<S> {
    pub fn new(sound: Sound, play_times: u8) -> Self {
        PlaySound {
            to_play: PlaySoundMsg {
                sound,
                play_ctrl: PlaySoundCtrl::Interupt,
                interval: 0,
                play_times,
            },

            progress: Default::default(),
            status: Default::default(),
        }
    }
}

impl ToProtoMessage<V1> for PlaySound<ActionUpdateHead> {
    type Message = PlaySoundMsg;

    fn to_proto_message(&self) -> Result<Self::Message> {
        Ok(self.to_play.clone())
    }
}

impl ProtoAction<V1> for PlaySound<ActionUpdateHead> {
    const TARGET: Option<Receiver> = DEFAULT_TARGET;
    type Update = PlaySoundUpdate;

    fn apply_state(&mut self, state: ActionState) -> Result<()> {
        self.status.state = state;
        Ok(())
    }

    fn apply_update(&mut self, update: (ActionUpdateHead, Self::Update)) -> Result<bool> {
        self.status = update.0;
        self.progress = update.1;

        Ok(self.status.is_completed())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PlaySoundCtrl {
    Stop = 0,
    Interupt = 1,
    Mixed = 2,
    Ignored = 3,
}

impl_v1_action_update!(PlaySoundUpdate, CMD_SET_CTRL, 0xb4);

#[derive(Debug, Default)]
pub struct PlaySoundUpdate {
    pub reserved: u8,
    pub sound_id: u32,
}

impl Deserialize<V1> for PlaySoundUpdate {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 4);

        Ok(Self {
            reserved: 0,
            sound_id: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
        })
    }
}

impl_v1_msg!(PlaySoundMsg, CMD_SET_CTRL, 0xb3);

#[derive(Debug, Clone)]
pub struct PlaySoundMsg {
    pub sound: Sound,
    pub play_ctrl: PlaySoundCtrl,
    pub interval: u16,
    pub play_times: u8,
}

impl Serialize<V1> for PlaySoundMsg {
    const SIZE_HINT: usize = 8;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.sound as u32)?;
        w.write_le(self.play_ctrl as u8)?;
        w.write_le(self.interval)?;
        w.write_le(self.play_times)?;
        Ok(())
    }
}
