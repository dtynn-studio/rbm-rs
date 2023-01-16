use std::sync::Arc;

use crate::{
    client::Client,
    module::{
        armor::Armor,
        battery::EPBattery,
        blaster::Blaster,
        camera::EPCamera,
        chassis::Chassis,
        common::{EPCommon, RobotMode},
        dds::DDS,
        gimbal::Gimbal,
        gripper::Gripper,
        led::EPLed,
        robotic_arm::RoboticArm,
        sensor::{adaptor::Adaptor, distance::Distance},
        servo::Servo,
        uart::Uart,
        vision::Vision,
    },
    proto::{v1::V1, Codec},
    Result,
};

// TODO: heartbeat
pub struct RobotMasterEP<CODEC: Codec, C: Client<CODEC>> {
    pub client: Arc<C>,
    pub common: EPCommon<CODEC, C>,
    pub chassis: Chassis<CODEC, C>,
    pub gimbal: Gimbal<CODEC, C>,
    pub camera: EPCamera<CODEC, C>,
    pub blaster: Blaster<CODEC, C>,
    pub vision: Vision<CODEC, C>,
    pub dds: DDS<CODEC, C>,
    pub led: EPLed<CODEC, C>,
    pub battery: EPBattery<CODEC, C>,
    pub servo: Servo<CODEC, C>,
    pub robotic_arm: RoboticArm<CODEC, C>,
    pub distance: Distance<CODEC, C>,
    pub sensor_adaptor: Adaptor<CODEC, C>,
    pub gripper: Gripper<CODEC, C>,
    pub armor: Armor<CODEC, C>,
    pub uart: Uart<CODEC, C>,
}

impl<C: Client<V1>> RobotMasterEP<V1, C> {
    pub fn new(client: Arc<C>) -> Result<Self> {
        let common = EPCommon::new(client.clone())?;
        let chassis = Chassis::new(client.clone())?;
        let gimbal = Gimbal::new(client.clone())?;
        let camera = EPCamera::new(client.clone())?;
        let blaster = Blaster::new(client.clone())?;
        let vision = Vision::new(client.clone())?;
        let dds = DDS::new(client.clone())?;
        let led = EPLed::new(client.clone())?;
        let battery = EPBattery::new(client.clone())?;
        let servo = Servo::new(client.clone())?;
        let robotic_arm = RoboticArm::new(client.clone())?;
        let distance = Distance::new(client.clone())?;
        let sensor_adaptor = Adaptor::new(client.clone())?;
        let gripper = Gripper::new(client.clone())?;
        let armor = Armor::new(client.clone())?;
        let uart = Uart::new(client.clone())?;

        let mut robot = Self {
            client,
            common,
            chassis,
            gimbal,
            camera,
            blaster,
            vision,
            dds,
            led,
            battery,
            servo,
            robotic_arm,
            distance,
            sensor_adaptor,
            gripper,
            armor,
            uart,
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
