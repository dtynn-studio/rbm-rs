use std::io::Cursor;

use crate::{
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize,
    },
    util::ordered::ReadOrderedExt,
    Result,
};

#[derive(Debug, Default)]
pub struct Battery {
    pub adc_value: u16,
    pub temperature: i16,
    pub current: i32,
    pub percent: u8,
}

impl Deserialize<V1> for Battery {
    fn de(buf: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(buf);

        let adc_value = reader.read_le()?;
        let temperature = reader.read_le()?;
        let current = reader.read_le()?;
        let percent = reader.read_le()?;

        Ok(Battery {
            adc_value,
            temperature,
            current,
            percent,
        })
    }
}

impl_v1_sub_self!(Battery);
