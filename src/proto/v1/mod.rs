use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{Codec, ProtoMessage, Raw};
use crate::{
    ensure_buf_size,
    util::algo::{crc16_calc, crc8_calc},
    Error, Result,
};

pub mod action;
pub mod cset;
mod util;

pub use util::*;

const RM_SDK_FIRST_SEQ_ID: u16 = 10000;
const RM_SDK_LAST_SEQ_ID: u16 = 20000;

const CMD_SEQ_MOD: u64 = (RM_SDK_LAST_SEQ_ID - RM_SDK_FIRST_SEQ_ID) as u64;

const MSG_HEADER_SIZE: usize = 13;
const MSG_MAGIN_NUM: u8 = 0x55;

pub type Sender = u8;
pub type Receiver = u8;
pub type Ident = (u8, u8);
pub type Seq = u16;

pub struct V1;

impl Codec for V1 {
    type Sender = Sender;
    type Receiver = Receiver;
    type Ident = Ident;
    type Seq = Seq;

    fn pack_msg<M: ProtoMessage<Self>>(
        sender: Self::Sender,
        receiver: Self::Receiver,
        seq: Self::Seq,
        msg: &M,
        need_ack: bool,
    ) -> Result<Vec<u8>> {
        let id = (M::IDENT, seq);
        let size = MSG_HEADER_SIZE + M::SIZE_HINT;

        let mut buf = Vec::with_capacity(size);
        buf.push(MSG_MAGIN_NUM);
        buf.push((size & 0xff) as u8);
        buf.push(((size >> 8) & 0x3 | 4) as u8);
        // crc header
        buf.push(crc8_calc(&buf[0..3], None));
        buf.push(sender);
        buf.push(receiver);
        buf.push((id.1 & 0xff) as u8);
        buf.push(((id.1 >> 8) & 0xff) as u8);

        // attri
        // is_ask should be recognized as resp, so attri here is always 0
        buf.push((need_ack as u8) << 5);

        // encode proto
        buf.push(id.0 .0);
        buf.push(id.0 .1);

        let mut writer = Cursor::new(&mut buf);
        msg.ser(&mut writer)?;

        // crc msg
        let crc_msg = crc16_calc(&buf[..], None).to_le_bytes();
        buf.push(crc_msg[0]);
        buf.push(crc_msg[1]);
        Ok(buf)
    }

    fn unpack_raw(buf: &[u8]) -> Result<(Raw<V1>, usize)> {
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
        let is_ack = buf[8] & 0x80 != 0;

        Ok((
            Raw {
                sender: buf[4],
                receiver: buf[5],
                is_ack,
                id: (buf[9], buf[10]),
                seq: ((buf[7] as u16) << 8) | buf[6] as u16,
                raw_data: (&buf[11..size - 2]).into(),
            },
            size,
        ))
    }
}

#[derive(Default)]
pub struct CmdSequence(AtomicU64);

impl CmdSequence {
    pub fn next(&self) -> Seq {
        let next = self.0.fetch_add(1, Ordering::Relaxed);
        RM_SDK_FIRST_SEQ_ID + (next % CMD_SEQ_MOD) as u16
    }
}
