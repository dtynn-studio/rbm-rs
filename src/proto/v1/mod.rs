use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{
    Codec, CodecCtx, DussMBAck, DussMBType, Message, RM_SDK_FIRST_ACTION_ID, RM_SDK_FIRST_SEQ_ID,
    RM_SDK_LAST_ACTION_ID, RM_SDK_LAST_SEQ_ID,
};
use crate::{
    algo::{crc16_calc, crc8_calc},
    ensure_buf_size, Error, Result,
};

pub mod camera;
pub mod ctrl;
pub mod gimbal;
pub mod gripper;
pub mod normal;
pub mod subscribe;
pub mod vision;

const MSG_HEADER_SIZE: usize = 13;
const MSG_MAGIN_NUM: u8 = 0x55;

pub type V1Ident = (u8, u8);

#[derive(Debug, Clone, Copy)]
pub struct V1Ctx {
    pub sender: u8,
    pub receiver: u8,
    need_ack: DussMBAck,
    is_ack_: bool,
}

impl CodecCtx for V1Ctx {
    fn need_ack(&self) -> DussMBAck {
        self.need_ack
    }

    fn is_ask(&self) -> bool {
        self.is_ack_
    }
}

pub struct V1 {
    cmd_seq: AtomicU64,
    action_seq: AtomicU64,
}

impl Default for V1 {
    fn default() -> Self {
        V1 {
            cmd_seq: AtomicU64::new(0),
            action_seq: AtomicU64::new(0),
        }
    }
}

const CMD_SEQ_MOD: u64 = (RM_SDK_LAST_SEQ_ID - RM_SDK_FIRST_SEQ_ID) as u64;
const ACTION_SEQ_MOD: u64 = (RM_SDK_LAST_ACTION_ID - RM_SDK_FIRST_ACTION_ID) as u64;

impl Codec for V1 {
    type Ident = V1Ident;
    type Seq = u16;
    type Ctx = V1Ctx;

    fn next_cmd_seq(&self) -> Self::Seq {
        let next = self.cmd_seq.fetch_add(1, Ordering::Relaxed);
        let seq = RM_SDK_FIRST_SEQ_ID + (next % CMD_SEQ_MOD) as u16;
        seq
    }

    fn next_action_seq(&self) -> Self::Seq {
        let next = self.action_seq.fetch_add(1, Ordering::Relaxed);
        let seq = RM_SDK_FIRST_ACTION_ID + (next % ACTION_SEQ_MOD) as u16;
        seq
    }

    fn ctx<M: Message<Ident = Self::Ident>>(
        sender: u8,
        receiver: u8,
        need_ack: Option<DussMBAck>,
    ) -> Self::Ctx {
        V1Ctx {
            sender,
            receiver,
            need_ack: need_ack.unwrap_or_else(|| {
                if M::CMD_TYPE == DussMBType::Push {
                    DussMBAck::No
                } else {
                    DussMBAck::Finish
                }
            }),
            is_ack_: false,
        }
    }

    fn pack_msg<M: Message<Ident = Self::Ident>>(
        &self,
        ctx: Self::Ctx,
        msg: M,
        seq: Self::Seq,
    ) -> Result<Vec<u8>> {
        let id = (M::IDENT, seq);
        let size = MSG_HEADER_SIZE + msg.size();

        let mut buf = vec![0u8; size];
        buf[0] = MSG_MAGIN_NUM;
        buf[1] = (size & 0xff) as u8;
        buf[2] = ((size >> 8) & 0x3 | 4) as u8;
        // crc header
        buf[3] = crc8_calc(&buf[0..3], None);
        buf[4] = ctx.sender;
        buf[5] = ctx.receiver;
        buf[6] = (id.1 & 0xff) as u8;
        buf[7] = ((id.1 >> 8) & 0xff) as u8;

        // attri
        // is_ask should be recognized as resp, so attri here is always 0
        buf[8] = (ctx.need_ack as u8) << 5;

        // encode proto
        buf[9] = id.0 .0;
        buf[10] = id.0 .1;

        let mut writer = Cursor::new(&mut buf[11..size - 2]);
        msg.ser(&mut writer)?;

        // crc msg
        let crc_msg = crc16_calc(&buf[..size - 2], None).to_le_bytes();
        buf[size - 2] = crc_msg[0];
        buf[size - 1] = crc_msg[1];
        Ok(buf)
    }

    fn unpack_raw(buf: &[u8]) -> Result<((Self::Ident, Self::Seq), Self::Ctx, &[u8], usize)> {
        ensure_buf_size!(buf, MSG_HEADER_SIZE, "raw msg header");
        if buf[0] != MSG_MAGIN_NUM {
            return Err(Error::InvalidData("invalid magic number".into()));
        }

        if crc8_calc(&buf[0..3], None) != buf[3] {
            return Err(Error::InvalidData("invalid crc header".into()));
        }

        let size = ((buf[2] as usize & 0x3) << 8) | buf[1] as usize;
        ensure_buf_size!(buf, size, "raw msg body");

        // TODO: check crc msg?
        let need_ack = ((buf[8] & 0x60) >> 5).try_into()?;

        Ok((
            ((buf[9], buf[10]), ((buf[7] as u16) << 8) | buf[6] as u16),
            V1Ctx {
                sender: buf[4],
                receiver: buf[5],
                is_ack_: buf[8] & 0x80 != 0,
                need_ack,
            },
            &buf[11..size - 2],
            size,
        ))
    }
}

macro_rules! impl_v1_cmd {
    ($name:ident, $resp:ty, $cid:literal) => {
        impl $crate::proto::Message for $name {
            type Ident = $crate::proto::v1::V1Ident;

            const IDENT: $crate::proto::v1::V1Ident = (CMD_SET, $cid);
        }

        impl $crate::proto::cmd::Command for $name {
            type Response = $resp;
        }
    };

    ($name:ident, $resp:ty, $cid:literal, $ctype:expr) => {
        impl $crate::proto::Message for $name {
            type Ident = $crate::proto::v1::V1Ident;

            const IDENT: $crate::proto::v1::V1Ident = (CMD_SET, $cid);
            const CMD_TYPE: $crate::proto::DussMBType = $ctype;
        }

        impl $crate::proto::cmd::Command for $name {
            type Response = $resp;
        }
    };
}

macro_rules! impl_v1_event {
    ($name:ident, $cid:literal) => {
        impl $crate::proto::Event for $name {
            type Ident = $crate::proto::v1::V1Ident;

            const IDENT: $crate::proto::v1::V1Ident = (CMD_SET, $cid);
        }
    };
}

macro_rules! impl_v1_action_response {
    ($name:ident, $field:ident) => {
        impl $crate::proto::ActionResponse for $name {
            fn progress(&self) -> &ActionProgress {
                &self.$field
            }
        }
    };
}

pub(self) use impl_v1_action_response;
pub(self) use impl_v1_cmd;
pub(self) use impl_v1_event;
