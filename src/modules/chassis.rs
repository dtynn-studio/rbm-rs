use crate::{
    conn::Client,
    proto::v1::{
        ctrl::{
            ChassisPwmFreq, ChassisPwmPercent, ChassisSpeedMode, ChassisStickOverlay,
            ChassisStickOverlayMode, SetWheelSpeed,
        },
        V1,
    },
    util::unit_convertor,
    Result,
};

pub struct Chassis {
    host: u8,
    client: Client<V1>,
}

impl Chassis {
    pub fn stick_overflow(&self, mode: ChassisStickOverlayMode) -> Result<()> {
        self.client
            .send_cmd(Some(self.host), ChassisStickOverlay { mode }, None)?;
        Ok(())
    }

    // TODO: delayed callback?
    pub fn drive_wheels(&self, w1: i16, w2: i16, w3: i16, w4: i16) -> Result<()> {
        let w1_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(w1)?;
        let w2_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(-w2)?;
        let w3_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(-w3)?;
        let w4_spd = unit_convertor::WHEEL_SPD_CONVERTOR.val2proto(w4)?;

        self.client.send_cmd(
            Some(self.host),
            SetWheelSpeed {
                w1_spd,
                w2_spd,
                w3_spd,
                w4_spd,
            },
            None,
        )?;

        Ok(())
    }

    pub fn drive_speed(&self, x: f32, y: f32, z: f32) -> Result<()> {
        let x_spd = unit_convertor::CHASSIS_SPD_X_CONVERTOR.val2proto(x)?;
        let y_spd = unit_convertor::CHASSIS_SPD_Y_CONVERTOR.val2proto(y)?;
        let z_spd = unit_convertor::CHASSIS_SPD_Z_CONVERTOR.val2proto(z)?;

        self.client.send_cmd(
            Some(self.host),
            ChassisSpeedMode {
                x_spd,
                y_spd,
                z_spd,
            },
            None,
        )?;

        Ok(())
    }

    pub fn set_pwm_value(&self, values: [Option<u16>; 6]) -> Result<()> {
        let mut mask = 0;
        let mut pwms = [0u16; 6];
        for i in 0..values.len() {
            if let Some(v) = values[i] {
                mask |= 1 << i;
                pwms[i] = unit_convertor::PWM_VALUE_CONVERTOR.val2proto(v)?;
            }
        }

        self.client
            .send_cmd(Some(self.host), ChassisPwmPercent { mask, pwms }, None)?;

        Ok(())
    }

    pub fn set_pwm_freq(&self, values: [Option<u16>; 6]) -> Result<()> {
        let mut mask = 0;
        let mut pwms = [0u16; 6];
        for i in 0..values.len() {
            if let Some(v) = values[i] {
                mask |= 1 << i;
                pwms[i] = unit_convertor::PWM_FREQ_CONVERTOR.val2proto(v)?;
            }
        }

        self.client
            .send_cmd(Some(self.host), ChassisPwmFreq { mask, pwms }, None)?;

        Ok(())
    }
}
