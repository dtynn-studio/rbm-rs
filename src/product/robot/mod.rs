use std::sync::Arc;

use crate::{
    client::Client,
    module::{
        blaster::Blaster,
        camera::EPCamera,
        chassis::Chassis,
        common::{EPCommon, RobotMode},
        dds::DDS,
        gimbal::Gimbal,
        vision::Vision,
    },
    proto::{v1::V1, Codec},
    Result,
};

// TODO: heartbeat
pub struct RobotMasterEP<CODEC: Codec, C: Client<CODEC>> {
    pub common: EPCommon<CODEC, C>,
    pub chassis: Chassis<CODEC, C>,
    pub gimbal: Gimbal<CODEC, C>,
    pub camera: EPCamera<CODEC, C>,
    pub blaster: Blaster<CODEC, C>,
    pub vision: Vision<CODEC, C>,
    pub dds: DDS<CODEC, C>,
}

impl<C: Client<V1>> RobotMasterEP<V1, C> {
    pub fn new(client: Arc<C>) -> Result<Self> {
        let common = EPCommon::new(client.clone())?;
        let chassis = Chassis::new(client.clone())?;
        let gimbal = Gimbal::new(client.clone())?;
        let camera = EPCamera::new(client.clone())?;
        let blaster = Blaster::new(client.clone())?;
        let vision = Vision::new(client.clone())?;
        let dds = DDS::new(client)?;

        let mut robot = Self {
            common,
            chassis,
            gimbal,
            camera,
            blaster,
            vision,
            dds,
        };

        robot.common.enable_sdk_mode(true)?;
        robot.reset()?;

        Ok(robot)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.dds.reset()?;
        self.common.set_robot_mode(RobotMode::Free)?;
        self.vision.reset()?;
        Ok(())
    }
}
