use super::impl_module;
use crate::{client::Client, module::common::constant::v1::DEFAULT_TARGET, proto::v1::V1, Result};

pub mod proto;
use proto::cmd::{NodeAdd, NodeReset};

impl_module!(DDS);

impl<C: Client<V1>> DDS<V1, C> {
    pub fn reset(&mut self) -> Result<()> {
        self.sub_node_reset()?;
        self.sub_add_node()
    }

    fn sub_node_reset(&mut self) -> Result<()> {
        let msg = NodeReset {
            node_id: self.client.host(),
        };

        self.client.send_cmd_sync(DEFAULT_TARGET, msg)?;

        Ok(())
    }

    fn sub_add_node(&mut self) -> Result<()> {
        self.client
            .send_cmd_sync(DEFAULT_TARGET, NodeAdd::new(self.client.host()))?;
        Ok(())
    }
}
