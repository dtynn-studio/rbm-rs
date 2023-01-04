use std::sync::Arc;

use crate::{
    client::Client,
    module::{
        common::{constant::v1::DEFAULT_TARGET, Common},
        dds::DDS,
        vision::Vision,
    },
    proto::{v1::V1, Codec},
    Result,
};

pub mod proto;
use proto::cmd;

// TODO: heartbeat
pub struct RobotMasterEP<CODEC: Codec, C: Client<CODEC>> {
    client: Arc<C>,

    pub common: Common<CODEC, C>,
    pub vision: Vision<CODEC, C>,
    pub dds: DDS<CODEC, C>,
}

impl<C: Client<V1>> RobotMasterEP<V1, C> {
    pub fn reset(&mut self) -> Result<()> {
        self.dds.reset()?;
        self.set_robot_mode(cmd::Mode::Free)?;
        self.vision.reset()?;
        Ok(())
    }

    pub fn set_robot_mode(&mut self, mode: cmd::Mode) -> Result<()> {
        self.client.send_cmd_sync(DEFAULT_TARGET, mode)?;
        Ok(())
    }

    pub fn robot_mode(&mut self) -> Result<cmd::Mode> {
        self.client.send_cmd_sync(DEFAULT_TARGET, cmd::GetMode)
    }
}
