use crate::{
    ensure_buf_size,
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize,
    },
    Result,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct PinInfo {
    pub io: u8,
    pub ad: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pin {
    pub port1: PinInfo,
    pub port2: PinInfo,
}

pub type Pinboard = [Pin; 6];

impl Deserialize<V1> for Pinboard {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 6 * 6);
        let mut pins: Pinboard = Default::default();

        for (i, chunk) in (buf[0..36]).chunks_exact(6).enumerate() {
            pins[i].port1.io = chunk[0];
            pins[i].port1.ad = chunk[2] as u16 | ((chunk[3] as u16) << 8);

            pins[i].port2.io = chunk[1];
            pins[i].port1.ad = chunk[4] as u16 | ((chunk[5] as u16) << 8);
        }

        Ok(pins)
    }
}

impl_v1_sub_self!(Pinboard);
