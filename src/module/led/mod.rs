use super::impl_module;
use crate::{
    client::Client, module::common::constant::v1::DEFAULT_TARGET, proto::v1::V1, Error, Result,
};

pub mod proto;
use proto::cmd::SetSystemLed;
pub use proto::cmd::{Comp, Effect};

// const DEFAULT_TARGET_V1: Option<Receiver> = Some(host2byte(24, 0));

impl_module!(EPLed);

impl<C: Client<V1>> EPLed<V1, C> {
    pub fn set_led(
        &mut self,
        comp: Comp,
        effect: Effect,
        r: Option<u8>,
        g: Option<u8>,
        b: Option<u8>,
        freq: u8,
    ) -> Result<()> {
        let mut cmd = SetSystemLed::new(
            comp,
            7,
            effect,
            r.unwrap_or(0),
            g.unwrap_or(0),
            b.unwrap_or(0),
        );

        match effect {
            Effect::Breath => {
                cmd.t1 = 1000;
                cmd.t2 = 1000;
            }

            Effect::Flash => {
                let t = 500 / (freq.max(1) as i16);
                cmd.t1 = t;
                cmd.t2 = t;
            }

            Effect::Scrolling => {
                cmd.t1 = 30;
                cmd.t2 = 40;
                cmd.led_mask = 0x0f;
            }

            _ => {}
        }

        self.client.send_cmd_sync(DEFAULT_TARGET, cmd)?;
        Ok(())
    }

    pub fn set_gimbal_led(
        &mut self,
        comp: Comp,
        effect: Effect,
        r: Option<u8>,
        g: Option<u8>,
        b: Option<u8>,
        leds: Option<&[u8]>,
    ) -> Result<()> {
        if !matches!(
            comp,
            Comp::ALL | Comp::TOP_ALL | Comp::TOP_LEFT | Comp::TOP_RIGHT
        ) {
            return Err(Error::InvalidData(
                format!("invalid comp {:?}", comp).into(),
            ));
        }

        if !matches!(effect, Effect::On | Effect::Off) {
            return Err(Error::InvalidData(
                format!("invalid effect {:?}", effect).into(),
            ));
        }

        let mut led_mask = 0;
        for led_idx in leds.unwrap_or(&[0, 1, 2, 3]) {
            let idx = *led_idx;
            if idx > 7 {
                return Err(Error::InvalidData(
                    format!("invalid led index {:?}", led_idx).into(),
                ));
            }
            led_mask |= 1 << idx;
        }

        let mut cmd = SetSystemLed::new(
            comp,
            7,
            effect,
            r.unwrap_or(255),
            g.unwrap_or(255),
            b.unwrap_or(255),
        );
        cmd.led_mask = led_mask;

        self.client.send_cmd_sync(DEFAULT_TARGET, cmd)?;
        Ok(())
    }
}
