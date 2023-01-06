use std::sync::Arc;

use crate::{
    client::Client,
    module::{
        chassis::Chassis,
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
    pub chassis: Chassis<CODEC, C>,
    pub vision: Vision<CODEC, C>,
    pub dds: DDS<CODEC, C>,
}

impl<C: Client<V1>> RobotMasterEP<V1, C> {
    pub fn new(client: Arc<C>) -> Result<Self> {
        let common = Common::new(client.clone())?;
        let chassis = Chassis::new(client.clone())?;
        let vision = Vision::new(client.clone())?;
        let dds = DDS::new(client.clone())?;

        let mut robot = Self {
            client,
            common,
            chassis,
            vision,
            dds,
        };

        robot.common.enable_sdk_mode(true)?;
        robot.reset()?;

        Ok(robot)
    }

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
