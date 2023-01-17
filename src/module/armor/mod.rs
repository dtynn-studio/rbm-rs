use std::sync::Arc;

use super::{impl_module, SubEventChanWithSubscription};
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::{chan::Rx, host2byte, unit_convertor},
    Result,
};

pub const V1_HOST: Option<Receiver> = Some(host2byte(24, 1));

pub mod proto;
use proto::cmd::SetParam;
pub use proto::{
    sub::{ArmorHit, IRHit},
    ArmorCompMask,
};

impl_module!(Armor, ~armor_hit_chan: SubEventChanWithSubscription<ArmorHit, CODEC>, ~ir_hit_chan: SubEventChanWithSubscription<IRHit, CODEC>);

impl<C: Client<V1>> Armor<V1, C> {
    pub fn set_hit_sensitive(&mut self, comp_mask: ArmorCompMask, sensitive: f32) -> Result<()> {
        let sens: f32 = unit_convertor::ARMOR_SENSITIVE_K_CONVERTOR.val2proto(sensitive)?;
        let k = 1.5 - sens / 10.0;
        let param = SetParam {
            armor_mask: comp_mask as u8,
            voice_energy_en: 500,
            voice_energy_ex: 300,
            voice_len_max: 50,
            voice_len_min: 13,
            voice_len_silence: 6,
            voice_peak_count: 1,
            voice_peak_min: (160.0 * k) as u16,
            voice_peak_ave: (180.0 * k) as u16,
            voice_peak_final: (200.0 * k) as u16,
        };

        self.client.send_cmd_sync(V1_HOST, param)?;

        Ok(())
    }

    pub fn sub_armor_hit_event(&mut self) -> Result<()> {
        if let Some(tx) = self.armor_hit_chan.0.tx.take() {
            let sub = self.client.subscribe_event(tx)?;
            self.armor_hit_chan.1.replace(sub);
        }

        Ok(())
    }

    pub fn armor_hit_rx(&self) -> &Arc<Rx<ArmorHit>> {
        &self.armor_hit_chan.0.rx
    }

    pub fn sub_ir_hit_event(&mut self) -> Result<()> {
        if let Some(tx) = self.ir_hit_chan.0.tx.take() {
            let sub = self.client.subscribe_event(tx)?;
            self.ir_hit_chan.1.replace(sub);
        }

        Ok(())
    }

    pub fn ir_hit_rx(&self) -> &Arc<Rx<IRHit>> {
        &self.ir_hit_chan.0.rx
    }
}
