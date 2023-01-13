use super::impl_module;
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
    Result,
};

pub mod proto;
use proto::cmd::{Fire, SetLed};
pub use proto::cmd::{FireType, LedEffect};

pub const V1_HOST: Option<Receiver> = Some(host2byte(1, 0));

impl_module!(Blaster);

impl<C: Client<V1>> Blaster<V1, C> {
    pub fn fire(&mut self, typ: FireType, times: u8) -> Result<()> {
        self.client.send_cmd_sync(V1_HOST, Fire::new(typ, times))?;
        Ok(())
    }

    pub fn set_led(&mut self, effect: LedEffect, brightness: u8, times: u8) -> Result<()> {
        let cmd = SetLed::new(effect, brightness, brightness, brightness, times);
        self.client.send_cmd_sync(V1_HOST, cmd)?;
        Ok(())
    }
}
