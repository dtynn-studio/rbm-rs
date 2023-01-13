use std::io::{Cursor, Write};

use super::SensorPort;
use crate::{
    ensure_ok,
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, V1},
        Deserialize, Serialize,
    },
    util::ordered::ReadOrderedExt,
    Result,
};

impl_v1_cmd!(SensorTarget, SensorData, CMD_SET_CTRL, 0xf0);

#[derive(Debug, Clone, Copy)]
pub struct SensorTarget(SensorPort);

impl From<SensorPort> for SensorTarget {
    fn from(val: SensorPort) -> Self {
        SensorTarget(val)
    }
}

impl Serialize<V1> for SensorTarget {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.0 as u8]).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct SensorData {
    pub port: u8,
    pub adc: u16,
    pub io: u8,
    pub time: u32,
}

impl Deserialize<V1> for SensorData {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        let mut reader = Cursor::new(buf);
        let port = reader.read_le()?;
        let adc = reader.read_le()?;
        let io = reader.read_le()?;
        let time = reader.read_le()?;

        Ok(SensorData {
            port,
            adc,
            io,
            time,
        })
    }
}
