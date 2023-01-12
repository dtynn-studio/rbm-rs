use tracing::trace;

use super::{impl_module, V1ActionReturn};
use crate::{
    client::Client,
    proto::{
        v1::{action::ActionUpdateHead, Receiver, V1},
        ProtoAction,
    },
    util::host2byte,
    Result,
};

pub mod constant;
use constant::v1::{Sound, DEFAULT_TARGET};

pub mod proto;
pub use proto::cmd::RobotMode;
use proto::{
    action::PlaySound,
    cmd::{EnableSdkMode, GetProductVersion, GetRobotMode, GetSN},
};

impl_module!(EPCommon);

const EP_COMMON_TARGET_V1: Option<Receiver> = Some(host2byte(8, 1));

impl<C: Client<V1>> EPCommon<V1, C> {
    pub fn version(&mut self) -> Result<(u8, u8, u16)> {
        let resp = self
            .client
            .send_cmd_sync(EP_COMMON_TARGET_V1, GetProductVersion::default())?;

        Ok((resp.major, resp.minor, resp.patch))
    }

    pub fn sn(&mut self) -> Result<String> {
        let resp = self
            .client
            .send_cmd_sync(EP_COMMON_TARGET_V1, GetSN::default())?;

        Ok(resp.sn)
    }

    pub fn enable_sdk_mode(&mut self, enable: bool) -> Result<()> {
        self.client
            .send_cmd_sync(DEFAULT_TARGET, EnableSdkMode::from(enable))?;
        Ok(())
    }

    pub fn set_robot_mode(&mut self, mode: RobotMode) -> Result<()> {
        self.client.send_cmd_sync(DEFAULT_TARGET, mode)?;
        Ok(())
    }

    pub fn robot_mode(&mut self) -> Result<RobotMode> {
        self.client.send_cmd_sync(DEFAULT_TARGET, GetRobotMode)
    }

    pub fn play_sound(&mut self, sound: Sound, play_times: u8) -> Result<()> {
        let (mut action, rx) = self.action_play_sound(sound, play_times)?;

        while let Some(update) = rx.recv() {
            let done = action.apply_update(update)?;
            trace!("play sound: {:?}", action.progress);
            if done {
                break;
            }
        }

        Ok(())
    }

    pub fn action_play_sound(
        &mut self,
        sound: Sound,
        play_times: u8,
    ) -> Result<V1ActionReturn<PlaySound<ActionUpdateHead>>> {
        let mut action = PlaySound::new(sound, play_times);

        let rx = self.client.send_action(None, &mut action)?;

        Ok((action, rx))
    }
}
