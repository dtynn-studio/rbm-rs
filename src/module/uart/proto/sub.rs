use crate::{
    ensure_buf_size,
    proto::{
        v1::{cset::CMD_SET_CTRL, Ident, V1},
        Deserialize, ProtoPush,
    },
    Result,
};

#[derive(Debug, Clone)]
pub struct SerialData(pub Option<Vec<u8>>);

impl Deserialize<V1> for SerialData {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 4);
        let data_len = ((buf[2] as usize) << 8) | buf[3] as usize;
        Ok(if buf[1] == 1 && buf.len() == data_len + 4 {
            Self(Some(buf[4..].to_owned()))
        } else {
            Self(None)
        })
    }
}

impl ProtoPush<V1> for SerialData {
    const IDENT: Ident = (CMD_SET_CTRL, 0xc1);
}
