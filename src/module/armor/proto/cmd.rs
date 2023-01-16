use std::io::Write;

use crate::{
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, RetOK, V1},
        Serialize,
    },
    util::ordered::WriteOrderedExt,
    Result,
};

#[derive(Debug)]
pub struct SetParam {
    pub armor_mask: u8,
    pub voice_energy_en: u16,
    pub voice_energy_ex: u16,
    pub voice_len_max: u16,
    pub voice_len_min: u16,
    pub voice_len_silence: u16,
    pub voice_peak_count: u16,
    pub voice_peak_min: u16,
    pub voice_peak_ave: u16,
    pub voice_peak_final: u16,
}

impl_v1_cmd!(SetParam, RetOK, CMD_SET_CTRL, 0x7);

impl Serialize<V1> for SetParam {
    const SIZE_HINT: usize = 19;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_le(self.armor_mask)?;
        w.write_le(self.voice_energy_en)?;
        w.write_le(self.voice_energy_ex)?;
        w.write_le(self.voice_len_max)?;
        w.write_le(self.voice_len_min)?;
        w.write_le(self.voice_len_silence)?;
        w.write_le(self.voice_peak_count)?;
        w.write_le(self.voice_peak_min)?;
        w.write_le(self.voice_peak_ave)?;
        w.write_le(self.voice_peak_final)?;
        Ok(())
    }
}
