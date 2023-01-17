use std::io::Cursor;

use crate::{
    ensure_buf_size,
    proto::{
        v1::{cset::CMD_SET_CTRL, Ident, V1},
        Deserialize, ProtoPush,
    },
    util::ordered::ReadOrderedExt,
    Result,
};

#[derive(Debug)]
pub struct AIEventItem {
    pub id: u8,
    pub x: u16,
    pub y: u8,
    pub w: u16,
    pub h: u8,
    pub c: u8,
}

#[derive(Debug)]
pub struct AIEvent(pub Option<Vec<AIEventItem>>);

impl Deserialize<V1> for AIEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 15);
        let num = (buf.len() - 15) / 8;
        Ok(AIEvent(if num == 0 {
            None
        } else {
            let mut items = Vec::with_capacity(num);
            let mut reader = Cursor::new(&buf[13..]);
            let id = reader.read_le()?;
            let x = reader.read_le()?;
            let y = reader.read_le()?;
            let w = reader.read_le()?;
            let h = reader.read_le()?;
            let c = reader.read_le()?;

            items.push(AIEventItem { id, x, y, w, h, c });
            Some(items)
        }))
    }
}

impl ProtoPush<V1> for AIEvent {
    const IDENT: Ident = (CMD_SET_CTRL, 0xea);
}
