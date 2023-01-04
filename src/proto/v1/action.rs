use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use byteorder::WriteBytesExt;

use super::{Ident, Seq, V1};
use crate::{
    ensure_buf_size,
    proto::{ActionState, Deserialize, ProtoCommand, ProtoMessage, ProtoPush, Serialize},
    Result, RetCode,
};

const RM_SDK_FIRST_ACTION_ID: u16 = 1;
const RM_SDK_LAST_ACTION_ID: u16 = 255;

const ACTION_SEQ_MOD: u64 = (RM_SDK_LAST_ACTION_ID - RM_SDK_FIRST_ACTION_ID) as u64;

#[derive(Default)]
pub struct ActionSequence(AtomicU64);

impl ActionSequence {
    pub fn next(&self) -> Seq {
        let next = self.0.fetch_add(1, Ordering::Relaxed);
        RM_SDK_FIRST_ACTION_ID + (next % ACTION_SEQ_MOD) as u16
    }
}

#[derive(Debug)]
pub struct ActionResponse {
    pub retcode: RetCode,
    pub acception: Option<u8>,
}

impl From<ActionResponse> for ActionState {
    fn from(v: ActionResponse) -> Self {
        match (v.retcode, v.acception) {
            (RetCode(0), Some(0)) => ActionState::Started,
            (RetCode(0), Some(1)) => ActionState::Rejected,
            (RetCode(0), Some(2)) => ActionState::Succeeded,
            _ => ActionState::Failed,
        }
    }
}

impl Deserialize<V1> for ActionResponse {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let retcode: RetCode = buf[0].into();
        let acception = if retcode.is_ok() {
            ensure_buf_size!(buf, 2);
            Some(buf[1])
        } else {
            None
        };

        Ok(Self { retcode, acception })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ActionCtrl {
    Start = 0,
    Cancel = 1,
}

impl Default for ActionCtrl {
    fn default() -> Self {
        Self::Start
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ActionUpdateFreq {
    OneHZ = 0,
    FiveHZ = 1,
    TenHz = 2,
}

impl Default for ActionUpdateFreq {
    fn default() -> Self {
        Self::TenHz
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActionConfig {
    pub freq: ActionUpdateFreq,
    pub ctrl: ActionCtrl,
}

#[derive(Debug, Clone, Copy)]
pub struct ActionHead {
    pub id: u8,
    pub cfg: ActionConfig,
}

impl Serialize<V1> for ActionHead {
    const SIZE_HINT: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.id)?;
        w.write_u8(self.cfg.ctrl as u8 | (self.cfg.freq as u8) << 2)?;
        Ok(())
    }
}

pub const ACTION_UPDATE_HEAD_SIZE: usize = 3;

#[derive(Debug)]
pub struct ActionUpdateHead {
    pub percent: u8,
    pub error_reason: u8,
    pub state: ActionState,
}

impl Default for ActionUpdateHead {
    fn default() -> Self {
        Self {
            percent: 0,
            error_reason: 0,
            state: ActionState::Idle,
        }
    }
}

impl ActionUpdateHead {
    pub fn is_completed(&self) -> bool {
        self.percent == 100 || self.state.is_completed()
    }
}

impl Deserialize<V1> for (Seq, ActionUpdateHead) {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, ACTION_UPDATE_HEAD_SIZE);
        Ok((
            buf[0] as u16,
            ActionUpdateHead {
                percent: buf[1],
                error_reason: buf[2] >> 2 & 0x03,
                state: (buf[2] & 0x03).try_into()?,
            },
        ))
    }
}

impl<T: ProtoMessage<V1>> Serialize<V1> for (ActionHead, T) {
    const SIZE_HINT: usize = T::SIZE_HINT + ActionHead::SIZE_HINT;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        self.0.ser(w)?;
        self.1.ser(w)
    }
}

impl<T: ProtoMessage<V1>> ProtoMessage<V1> for (ActionHead, T) {
    const IDENT: Ident = T::IDENT;
}

impl<T: ProtoMessage<V1>> ProtoCommand<V1> for (ActionHead, T) {
    type Resp = ActionResponse;
}

impl<T: Deserialize<V1>> Deserialize<V1> for ((Seq, ActionUpdateHead), T) {
    fn de(buf: &[u8]) -> Result<Self> {
        let (seq, head): (Seq, ActionUpdateHead) = Deserialize::de(buf)?;
        let update: T = Deserialize::de(&buf[ACTION_UPDATE_HEAD_SIZE..])?;
        Ok(((seq, head), update))
    }
}
