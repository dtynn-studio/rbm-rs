use super::impl_module;
use crate::{
    client::{Client, Subscription},
    proto::v1::{Receiver, V1},
    util::{chan::Tx, host2byte},
    Result,
};

pub mod proto;
use proto::cmd::AIInit;
pub use proto::sub::AIEvent;

pub const V1_HOST: Option<Receiver> = Some(host2byte(15, 1));

impl_module!(EPAI);

impl<C: Client<V1>> EPAI<V1, C> {
    pub fn init_module(&mut self) -> Result<()> {
        let cmd = AIInit::default();
        self.client.send_cmd_sync(Some(host2byte(9, 2)), cmd)?;
        Ok(())
    }

    pub fn sub_ai_event(&mut self, tx: Tx<AIEvent>) -> Result<Box<dyn Subscription<V1>>> {
        self.client.subscribe_event(tx)
    }
}
