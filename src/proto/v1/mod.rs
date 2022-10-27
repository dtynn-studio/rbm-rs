use std::io::Cursor;

use super::{Codec, Command, DussMBAck, DussMBType, Msg};
use crate::{
    algo::{crc16_calc, crc8_calc},
    ensure_buf_size, Error, Result,
};

pub mod normal;

const MSG_HEADER_SIZE: usize = 13;
const MSG_MAGIN_NUM: u8 = 0x55;

pub trait V1Proto {
    const IDENT: V1ProtoIdent;
    const CMD_TYPE: DussMBType = DussMBType::Req;
}

pub type V1ProtoIdent = (u8, u8);

#[derive(Debug, Clone, Copy)]
pub struct V1MsgCtx {
    pub sender: u8,
    pub receiver: u8,
    pub need_ack: DussMBAck,
    pub is_ack: bool,
}

pub struct V1Msg<P>
where
    P: V1Proto + Command,
{
    pub sender: u8,
    pub receiver: u8,
    pub seq_id: u16,

    need_ack: DussMBAck,

    proto: P,
}

impl<P> Msg for V1Msg<P>
where
    P: V1Proto + Command,
{
    type Ident = (V1ProtoIdent, u16);
    type Ctx = V1MsgCtx;

    fn ident(&self) -> Self::Ident {
        (P::IDENT, self.seq_id)
    }

    fn ctx(&self) -> Self::Ctx {
        V1MsgCtx {
            sender: self.sender,
            receiver: self.receiver,
            need_ack: self.need_ack,
            is_ack: false,
        }
    }
}

pub struct V1;

impl<P> Codec<V1Msg<P>> for V1
where
    P: V1Proto + Command,
{
    fn pack_msg(msg: V1Msg<P>) -> Result<Vec<u8>> {
        let size = MSG_HEADER_SIZE + P::SIZE;
        let mut buf = vec![0u8; size];
        buf[0] = MSG_MAGIN_NUM;
        buf[1] = (size & 0xff) as u8;
        buf[2] = ((size >> 8) & 0x3 | 4) as u8;
        // crc header
        buf[3] = crc8_calc(&buf[0..3], None);
        buf[4] = msg.sender;
        buf[5] = msg.receiver;
        buf[6] = (msg.seq_id & 0xff) as u8;
        buf[7] = ((msg.seq_id >> 8) & 0xff) as u8;

        // attri
        // is_ask should be recognized as resp, so attri here is always 0
        buf[8] = (msg.need_ack as u8) << 5;

        // encode proto
        buf[9] = P::IDENT.0;
        buf[10] = P::IDENT.1;

        let mut writer = Cursor::new(&mut buf[11..size - 2]);
        msg.proto.ser(&mut writer)?;

        // crc msg
        let crc_msg = crc16_calc(&buf[..size - 2], None).to_le_bytes();
        buf[size - 2] = crc_msg[0];
        buf[size - 1] = crc_msg[1];
        Ok(buf)
    }

    fn unpack_raw(
        buf: &[u8],
    ) -> Result<(
        <V1Msg<P> as Msg>::Ident,
        <V1Msg<P> as Msg>::Ctx,
        &[u8],
        usize,
    )> {
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
            V1MsgCtx {
                sender: buf[4],
                receiver: buf[5],
                is_ack: buf[8] & 0x80 != 0,
                need_ack,
            },
            &buf[11..size - 2],
            size,
        ))
    }
}

macro_rules! impl_v1_cmd {
    (cmd: $name:ident, $resp:ty) => {
        impl $crate::proto::Command for $name {
            type Response = $resp;
        }
    };

    ($name:ident, $resp:ty, $cid:literal) => {
        impl $crate::proto::v1::V1Proto for $name {
            const IDENT: $crate::proto::v1::V1ProtoIdent = (CMD_SET, $cid);
        }

        impl_v1_cmd!(cmd: $name, $resp);
    };

    ($name:ident, $resp:ty, $cid:literal, $ctype:literal) => {
        impl $crate::proto::v1::V1Proto for $name {
            const IDENT: $crate::proto::v1::V1ProtoIdent = (CMD_SET, $cid);
            const CMD_TYPE: $crate::proto::DussMBType = $ctype;
        }

        impl_v1_cmd!(cmd: $name, $resp);
    };
}

pub(self) use impl_v1_cmd;
