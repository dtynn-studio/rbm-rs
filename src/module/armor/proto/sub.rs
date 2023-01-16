use std::io::Cursor;

use super::ArmorId;
use crate::{
    ensure_buf_size,
    proto::{
        v1::{cset::CMD_SET_CTRL, Ident, V1},
        Deserialize, ProtoPush,
    },
    util::{macros::impl_num_enums, ordered::ReadOrderedExt},
    Result,
};

impl_num_enums!(HitType, Water = 0, IR = 1,);

#[derive(Debug)]
pub struct ArmorHit {
    pub id: ArmorId,
    pub typ: HitType,
    pub mic_value: u16,
    pub acc_value: u16,
}

impl Deserialize<V1> for ArmorHit {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);
        let info_byte: u8 = reader.read_le()?;
        let id = ArmorId::try_from(info_byte >> 4)?;
        let typ = HitType::try_from(info_byte & 0x0f)?;
        let mic_value = reader.read_le()?;
        let acc_value = reader.read_le()?;

        Ok(ArmorHit {
            id,
            typ,
            mic_value,
            acc_value,
        })
    }
}

impl ProtoPush<V1> for ArmorHit {
    const IDENT: Ident = (CMD_SET_CTRL, 0x02);
}

pub struct IRHit {
    pub skill_id: u8,
    pub role_id: u8,
    pub recv_dev: u8,
    pub recv_ir_pin: u8,
}

impl Deserialize<V1> for IRHit {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 3);
        Ok(IRHit {
            skill_id: buf[0] >> 4,
            role_id: buf[0] & 0x0f,
            recv_dev: buf[1],
            recv_ir_pin: buf[2],
        })
    }
}

impl ProtoPush<V1> for IRHit {
    const IDENT: Ident = (CMD_SET_CTRL, 0x10);
}
