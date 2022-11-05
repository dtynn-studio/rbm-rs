use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use crate::{
    ensure_buf_size, ensure_ok,
    proto::{
        v1::{impl_v1_cmd, impl_v1_event},
        Deserialize, RetOK, Serialize,
    },
    Error, Result, RetCode,
};

const CMD_SET: u8 = 0x48;

impl_v1_cmd!(SubscribeAddNode, SubscribeAddNodeResp, 0x01);

#[derive(Debug)]
pub struct SubscribeAddNodeResp {
    pub pub_node_id: u8,
}

impl Deserialize for SubscribeAddNodeResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let retcode: RetCode = buf[0].into();
        if retcode.0 == 0 || retcode.0 == 0x50 {
            ensure_buf_size!(buf, 2);
            Ok(SubscribeAddNodeResp {
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
pub struct SubscribeAddNode {
    pub node_id: u8,
    pub sub_vision: u32,
}

impl Default for SubscribeAddNode {
    fn default() -> Self {
        Self {
            node_id: 0,
            sub_vision: 0x03000000,
        }
    }
}

impl Serialize for SubscribeAddNode {
    const SIZE: usize = 5;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id)?;
        w.write_u32::<LE>(self.sub_vision)?;
        Ok(())
    }
}

impl_v1_cmd!(SubNodeReset, RetOK, 0x02);

#[derive(Debug, Default)]
pub struct SubNodeReset {
    pub node_id: u8,
}

impl Serialize for SubNodeReset {
    const SIZE: usize = 1;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id).map_err(From::from)
    }
}

impl_v1_cmd!(DelMsg, RetOK, 0x04);

#[derive(Debug)]
pub struct DelMsg {
    pub sub_mode: u8,
    pub node_id: u8,
    pub msg_id: u8,
}

impl Serialize for DelMsg {
    const SIZE: usize = 3;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.sub_mode, self.node_id, self.msg_id])
            .map_err(From::from)
    }
}

impl_v1_cmd!(AddSubMsg, AddSubMsgResp, 0x03);

#[derive(Debug)]
pub struct AddSubMsgResp {
    pub pub_node_id: u8,
    pub ack_sub_mode: u8,
    pub ack_msg_id: u8,
    pub ack_err_uid_data: u64,
}

impl Deserialize for AddSubMsgResp {
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

#[derive(Debug)]
pub struct AddSubMsg {
    pub node_id: u8,
    pub msg_id: u8,
    pub timestamp: u8,
    pub stop_when_disconnect: u8,
    pub sub_mode: u8,
    pub sub_uid_list: Vec<u64>,
    pub sub_freq: u16,
}

impl Default for AddSubMsg {
    fn default() -> Self {
        Self {
            node_id: 0,
            msg_id: 0,
            timestamp: 0,
            stop_when_disconnect: 0,
            sub_mode: 0,
            sub_uid_list: vec![],
            sub_freq: 1,
        }
    }
}

impl Serialize for AddSubMsg {
    const SIZE: usize = 7;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id)?;
        w.write_u8(self.msg_id)?;
        w.write_u8((self.timestamp & 0x1) | (self.stop_when_disconnect & 0x2))?;
        w.write_u8(self.sub_mode)?;
        w.write_u8(self.sub_uid_list.len() as u8)?;
        for uid in self.sub_uid_list.iter() {
            w.write_u64::<LE>(*uid)?;
        }

        w.write_u16::<LE>(self.sub_freq)?;

        Ok(())
    }

    fn size(&self) -> usize {
        Self::SIZE + self.sub_uid_list.len() * 8
    }
}

impl_v1_event!(PushPeriodMsg, 0x8);

#[derive(Debug)]
pub struct PushPeriodMsg {
    pub sub_mode: u8,
    pub msg_id: u8,
    pub data: Vec<u8>,
}

impl Deserialize for PushPeriodMsg {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 2);
        Ok(Self {
            sub_mode: buf[0],
            msg_id: buf[1],
            data: buf[2..].to_owned(),
        })
    }
}
