use super::impl_module;
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
    Result,
};

pub mod constant;
use constant::v1::DEFAULT_TARGET;

pub mod proto;
use proto::cmd::{EnableSdkMode, GetProductVersion, GetSN};

impl_module!(Common);

const COMMON_TARGET_V1: Option<Receiver> = Some(host2byte(8, 1));

impl<C: Client<V1>> Common<V1, C> {
    pub fn version(&mut self) -> Result<(u8, u8, u16)> {
        let resp = self
            .client
            .send_cmd_sync(COMMON_TARGET_V1, GetProductVersion::default())?;

        Ok((resp.major, resp.minor, resp.patch))
    }

    pub fn sn(&mut self) -> Result<String> {
        let resp = self
            .client
            .send_cmd_sync(COMMON_TARGET_V1, GetSN::default())?;

        Ok(resp.sn)
    }

    pub fn enable_sdk_mode(&mut self, enable: bool) -> Result<()> {
        self.client
            .send_cmd_sync(DEFAULT_TARGET, EnableSdkMode::from(enable))?;
        Ok(())
    }
}
