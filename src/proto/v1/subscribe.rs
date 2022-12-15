use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use super::{cset::CMD_SET_SUBSCRIBE, impl_v1_cmd, Ident, RetOK, V1};
use crate::{
    ensure_buf_size, ensure_ok,
    proto::{Deserialize, ProtoPush, Serialize},
    Error, Result, RetCode,
};

pub const PUSH_PERIOD_MSG_IDENT: Ident = (CMD_SET_SUBSCRIBE, 0x8);

impl_v1_cmd!(AddNode, AddNodeResp, CMD_SET_SUBSCRIBE, 0x01);

#[derive(Debug)]
pub struct AddNodeResp {
    pub pub_node_id: u8,
}

impl Deserialize<V1> for AddNodeResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let retcode: RetCode = buf[0].into();
        if retcode.0 == 0 || retcode.0 == 0x50 {
            ensure_buf_size!(buf, 2);
            Ok(AddNodeResp {
                pub_node_id: buf[1],
            })
        } else {
            Err(Error::NotOK {
                code: retcode,
                errcode: None,
                msg: None,
            })
        }
    }
}

#[derive(Debug)]
pub struct AddNode {
    pub node_id: u8,
    pub sub_vision: u32,
}

impl Default for AddNode {
    fn default() -> Self {
        Self {
            node_id: 0,
            sub_vision: 0x03000000,
        }
    }
}

impl Serialize<V1> for AddNode {
    const SIZE_HINT: usize = 5;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id)?;
        w.write_u32::<LE>(self.sub_vision)?;
        Ok(())
    }
}

impl_v1_cmd!(SubNodeReset, RetOK, CMD_SET_SUBSCRIBE, 0x02);

#[derive(Debug, Default)]
pub struct SubNodeReset {
    pub node_id: u8,
}

impl Serialize<V1> for SubNodeReset {
    const SIZE_HINT: usize = 1;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id).map_err(From::from)
    }
}

impl_v1_cmd!(SubMsg, SubMsgResp, CMD_SET_SUBSCRIBE, 0x03);

#[derive(Debug)]
pub struct SubMsgResp {
    pub pub_node_id: u8,
    pub ack_sub_mode: u8,
    pub ack_msg_id: u8,
    pub ack_err_uid_data: u64,
}

impl Deserialize<V1> for SubMsgResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 8);
        Ok(Self {
            pub_node_id: buf[1],
            ack_sub_mode: buf[2],
            ack_msg_id: buf[3],
            ack_err_uid_data: buf[4] as u64
                | (buf[5] as u64) << 8
                | (buf[6] as u64) << 16
                | (buf[7] as u64) << 24,
        })
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum SubFreq {
    OneHz = 1,
    FiveHz = 5,
    TenHz = 10,
    TwentyHz = 20,
    FiftyHz = 50,
}

#[derive(Debug)]
pub struct SubMsg {
    pub node_id: u8,
    pub msg_id: u8,
    pub timestamp: u8,
    pub stop_when_disconnect: u8,
    pub sub_mode: u8,
    pub sub_uid_list: Vec<u64>,
    pub sub_freq: SubFreq,
}

impl SubMsg {
    pub fn single(node_id: u8, msg_id: u8, freq: SubFreq, uid: u64) -> Self {
        Self {
            node_id,
            msg_id,
            sub_uid_list: vec![uid],
            sub_freq: freq,
            ..Default::default()
        }
    }
}

impl Default for SubMsg {
    fn default() -> Self {
        Self {
            node_id: 0,
            msg_id: 0,
            timestamp: 0,
            stop_when_disconnect: 0,
            sub_mode: 0,
            sub_uid_list: vec![],
            sub_freq: SubFreq::OneHz,
        }
    }
}

impl Serialize<V1> for SubMsg {
    const SIZE_HINT: usize = 7;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id)?;
        w.write_u8(self.msg_id)?;
        w.write_u8((self.timestamp & 0x1) | (self.stop_when_disconnect & 0x2))?;
        w.write_u8(self.sub_mode)?;
        w.write_u8(self.sub_uid_list.len() as u8)?;
        for uid in self.sub_uid_list.iter() {
            w.write_u64::<LE>(*uid)?;
        }

        w.write_u16::<LE>(self.sub_freq as u16)?;

        Ok(())
    }

    fn size(&self) -> usize {
        Self::SIZE_HINT + self.sub_uid_list.len() * 8
    }
}

impl_v1_cmd!(UnsubMsg, RetOK, CMD_SET_SUBSCRIBE, 0x04);

#[derive(Debug)]
pub struct UnsubMsg {
    pub sub_mode: u8,
    pub node_id: u8,
    pub msg_id: u8,
}

impl Serialize<V1> for UnsubMsg {
    const SIZE_HINT: usize = 3;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.sub_mode, self.node_id, self.msg_id])
            .map_err(From::from)
    }
}

pub struct PushPeriodMsg {
    pub sub_mode: u8,
    pub msg_id: u8,
    pub data: Vec<u8>,
}

impl Deserialize<V1> for PushPeriodMsg {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 2);

        Ok(Self {
            sub_mode: buf[0],
            msg_id: buf[1],
            data: (buf[2..]).to_owned(),
        })
    }
}

impl ProtoPush<V1> for PushPeriodMsg {
    const IDENT: Ident = PUSH_PERIOD_MSG_IDENT;
}

pub trait PushPeriodRequest {
    const UID: u64;

    type Push: Deserialize<V1>;
}
