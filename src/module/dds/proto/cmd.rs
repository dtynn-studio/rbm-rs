use std::io::Write;

use byteorder::{WriteBytesExt, LE};

use crate::{
    ensure_buf_size,
    proto::{
        v1::{cset::CMD_SET_SUBSCRIBE, impl_v1_cmd, RetOK, V1},
        Deserialize, Serialize,
    },
    Error, Result, RetCode,
};

impl_v1_cmd!(NodeAdd, NodeAddResp, CMD_SET_SUBSCRIBE, 0x01);

#[derive(Debug)]
pub struct NodeAddResp {
    pub pub_node_id: u8,
}

impl Deserialize<V1> for NodeAddResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let retcode: RetCode = buf[0].into();
        if retcode.0 == 0 || retcode.0 == 0x50 {
            ensure_buf_size!(buf, 2);
            Ok(NodeAddResp {
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
pub struct NodeAdd {
    pub node_id: u8,
    pub sub_vision: u32,
}

impl NodeAdd {
    pub fn new(node_id: u8) -> Self {
        NodeAdd {
            node_id,
            sub_vision: 0x03000000,
        }
    }
}

impl Serialize<V1> for NodeAdd {
    const SIZE_HINT: usize = 5;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id)?;
        w.write_u32::<LE>(self.sub_vision)?;
        Ok(())
    }
}

impl_v1_cmd!(NodeReset, RetOK, CMD_SET_SUBSCRIBE, 0x02);

#[derive(Debug, Default)]
pub struct NodeReset {
    pub node_id: u8,
}

impl Serialize<V1> for NodeReset {
    const SIZE_HINT: usize = 1;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.node_id).map_err(From::from)
    }
}
