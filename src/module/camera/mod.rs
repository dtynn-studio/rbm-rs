use super::impl_module;
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
    Result,
};

pub mod proto;
use proto::cmd::TakePhoto;

pub const V1_HOST: Option<Receiver> = Some(host2byte(1, 0));

impl_module!(EPCamera);

// TODO: live steram
impl<C: Client<V1>> EPCamera<V1, C> {
    pub fn take_photo(&mut self) -> Result<()> {
        self.client.send_cmd_sync(V1_HOST, TakePhoto)?;
        Ok(())
    }
}
