use tracing::trace;

use super::{impl_module, impl_v1_subscribe_meth_simple, V1ActionReturn, V1SubscribeReturn};
use crate::{
    client::Client,
    proto::{
        v1::{action::ActionUpdateHead, subscribe::SubFreq, Receiver, V1},
        ProtoAction,
    },
    util::{host2byte, unit_convertor},
    Result,
};

pub mod proto;
pub use proto::cmd::StickOverlayMode;
use proto::{
    action::Move,
    cmd::{SetPwmFreq, SetPwmPercent, SetSpeed, SetWheelSpeed},
    sub::{
        Attitude, ChassisMode, Esc, Imu, Position, PositionOriginMode, SaStatus, Sbus, Velocity,
    },
};

const CHASSIS_TARGET_V1: Option<Receiver> = Some(host2byte(3, 6));

impl_module!(Chassis);

impl<C: Client<V1>> Chassis<V1, C> {
    pub fn set_stick_overlay(&mut self, mode: StickOverlayMode) -> Result<()> {
        self.client.send_cmd_sync(CHASSIS_TARGET_V1, mode)?;
        Ok(())
    }

    pub fn set_wheel_speed(&mut self, w1: i16, w2: i16, w3: i16, w4: i16) -> Result<()> {
        let w1_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(w1)?;
        let w2_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(-w2)?;
        let w3_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(-w3)?;
        let w4_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(w4)?;

        self.client.send_cmd_sync(
            CHASSIS_TARGET_V1,
            SetWheelSpeed {
                w1_spd,
                w2_spd,
                w3_spd,
                w4_spd,
            },
        )?;

        Ok(())
    }

    pub fn set_speed(&mut self, x: f32, y: f32, z: f32) -> Result<()> {
        let x_spd = unit_convertor::CHASSIS_SPD_X_CONVERTOR.val2proto(x)?;
        let y_spd = unit_convertor::CHASSIS_SPD_Y_CONVERTOR.val2proto(y)?;
        let z_spd = unit_convertor::CHASSIS_SPD_Z_CONVERTOR.val2proto(z)?;

        self.client.send_cmd_sync(
            CHASSIS_TARGET_V1,
            SetSpeed {
                x_spd,
                y_spd,
                z_spd,
            },
        )?;

        Ok(())
    }

    pub fn set_pwm_percent(&mut self, values: [Option<u16>; 6]) -> Result<()> {
        let mut mask = 0;
        let mut pwms = [0u16; 6];
        for i in 0..values.len() {
            if let Some(v) = values[i] {
                mask |= 1 << i;
                pwms[i] = unit_convertor::PWM_VALUE_CONVERTOR.val2proto(v)?;
            }
        }

        self.client
            .send_cmd_sync(CHASSIS_TARGET_V1, SetPwmPercent { mask, pwms })?;

        Ok(())
    }

    pub fn set_pwm_freq(&mut self, values: [Option<u16>; 6]) -> Result<()> {
        let mut mask = 0;
        let mut pwms = [0u16; 6];
        for i in 0..values.len() {
            if let Some(v) = values[i] {
                mask |= 1 << i;
                pwms[i] = unit_convertor::PWM_FREQ_CONVERTOR.val2proto(v)?;
            }
        }

        self.client
            .send_cmd_sync(CHASSIS_TARGET_V1, SetPwmFreq { mask, pwms })?;

        Ok(())
    }

    pub fn action_move(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        xy_speed: Option<f32>,
        z_speed: Option<f32>,
    ) -> Result<V1ActionReturn<Move<ActionUpdateHead>>> {
        let mut action = Move::<ActionUpdateHead>::new(
            x,
            y,
            z,
            xy_speed.unwrap_or(0.5),
            z_speed.unwrap_or(30.0),
        );

        let rx = self.client.send_action(None, &mut action)?;

        Ok((action, rx))
    }

    // TODO: timeout?
    pub fn move_to(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        xy_speed: Option<f32>,
        z_speed: Option<f32>,
    ) -> Result<()> {
        let (mut action, rx) = self.action_move(x, y, z, xy_speed, z_speed)?;

        while let Some(update) = rx.recv() {
            let done = action.apply_update(update)?;
            trace!("move progress: {:?}", action.progress);
            if done {
                break;
            }
        }

        Ok(())
    }

    pub fn subscribe_position(
        &mut self,
        origin: PositionOriginMode,
        freq: Option<SubFreq>,
    ) -> Result<V1SubscribeReturn<Position>> {
        let (pos_rx, sub) = self.client.subscribe_period_push::<Position>(freq)?;
        Ok((Position::new(origin), pos_rx, sub))
    }

    impl_v1_subscribe_meth_simple!(Attitude);

    impl_v1_subscribe_meth_simple!(ChassisMode);

    impl_v1_subscribe_meth_simple!(Sbus);

    impl_v1_subscribe_meth_simple!(Velocity);

    impl_v1_subscribe_meth_simple!(Esc);

    impl_v1_subscribe_meth_simple!(Imu);

    impl_v1_subscribe_meth_simple!(SaStatus);
}
