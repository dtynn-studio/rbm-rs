use crate::{
    ensure_buf_size,
    module::common::constant::v1::Uid,
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize,
    },
    Error, Result,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Opened = 1,
    Closed = 2,
    Normal = 0,
}

impl Default for Status {
    fn default() -> Self {
        Self::Normal
    }
}

impl Deserialize<V1> for Status {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        Ok(match buf[0] {
            0 => Status::Normal,
            1 => Status::Opened,
            2 => Status::Closed,
            other => {
                return Err(Error::InvalidData(
                    format!("unknown status {}", other).into(),
                ))
            }
        })
    }
}

impl_v1_sub_self!(Status, Uid::Gripper as u64);
