use crate::{
    ensure_buf_size,
    module::common::constant::v1::Uid,
    proto::{
        v1::{impl_v1_sub_self, V1},
        Deserialize,
    },
    Result,
};

#[derive(Debug, Default)]
pub struct TOF {
    pub id: u8,
    pub direct: u8,
    pub flag: u8,
    pub distance: u16,
}

pub type Tofs = [TOF; 4];

impl Deserialize<V1> for Tofs {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 20);

        let mut tofs: Tofs = Default::default();
        for (i, chunk) in (buf[0..20]).chunks_exact(5).enumerate() {
            tofs[i].id = chunk[0];
            tofs[i].direct = chunk[1];
            tofs[i].flag = chunk[2];
            tofs[i].distance = ((chunk[3] as u16) << 8) | chunk[4] as u16;
        }

        Ok(tofs)
    }
}

impl_v1_sub_self!(Tofs, Uid::Tof as u64);
