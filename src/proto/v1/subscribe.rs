use super::{cset::CMD_SET_SUBSCRIBE, Ident, V1};
use crate::{
    ensure_buf_size,
    proto::{Deserialize, ProtoPush},
    Result,
};

pub const PUSH_PERIOD_MSG_IDENT: Ident = (CMD_SET_SUBSCRIBE, 0x8);

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
