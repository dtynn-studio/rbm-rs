use super::{impl_module, SubEventChanWithSubscription};
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
    Result,
};

pub mod proto;
use proto::{cmd::AIInit, sub::AIEvent};

pub const V1_HOST: Option<Receiver> = Some(host2byte(15, 1));

impl_module!(EPAI, ~ai_event_chan: SubEventChanWithSubscription<AIEvent, CODEC>);

impl<C: Client<V1>> EPAI<V1, C> {
    pub fn init_module(&mut self) -> Result<()> {
        let cmd = AIInit::default();
        self.client.send_cmd_sync(Some(host2byte(9, 2)), cmd)?;
        Ok(())
    }

    pub fn sub_ai_event(&mut self) -> Result<()> {
        if let Some(tx) = self.ai_event_chan.0.tx.take() {
            let sub = self.client.subscribe_event(tx)?;
            self.ai_event_chan.1.replace(sub);
        }

        Ok(())
    }
}
