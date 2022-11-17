use std::io::Cursor;

use super::{Codec, Message, Raw};
use crate::{
    ensure_buf_size,
    util::algo::{crc16_calc, crc8_calc},
    Error, Result,
};

const MSG_HEADER_SIZE: usize = 13;
const MSG_MAGIN_NUM: u8 = 0x55;

pub struct V1;

impl Codec for V1 {
    type Sender = u8;
    type Receiver = u8;
    type Ident = (u8, u8);
    type Seq = u16;

    fn pack_msg<M: Message<Self>>(
        sender: Self::Sender,
        receiver: Self::Receiver,
        seq: Self::Seq,
        msg: M,
        need_ack: bool,
    ) -> Result<Vec<u8>> {
        let id = (M::IDENT, seq);
        let size = MSG_HEADER_SIZE + msg.size();

        let mut buf = vec![0u8; size];
        buf[0] = MSG_MAGIN_NUM;
        buf[1] = (size & 0xff) as u8;
        buf[2] = ((size >> 8) & 0x3 | 4) as u8;
        // crc header
        buf[3] = crc8_calc(&buf[0..3], None);
        buf[4] = sender;
        buf[5] = receiver;
        buf[6] = (id.1 & 0xff) as u8;
        buf[7] = ((id.1 >> 8) & 0xff) as u8;

        // attri
        // is_ask should be recognized as resp, so attri here is always 0
        buf[8] = (need_ack as u8) << 5;

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
                raw_data: &buf[11..size - 2],
            },
            size,
        ))
    }
}
