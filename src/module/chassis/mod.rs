use tracing::trace;

use super::impl_module;
use crate::{
    client::{Client, Subscription},
    proto::{
        v1::{action::ActionUpdateHead, subscribe::SubFreq, Receiver, V1},
        ProtoAction,
    },
    util::{chan::Rx, host2byte},
    Result,
};

pub mod proto;
pub use proto::{
    action::{Move, MoveProgress, MoveUpdate},
    cmd::StickOverlayMode,
    subscribe::{Position, PositionOriginMode, PositionPush},
};

const CHASSIS_TARGET_V1: Option<Receiver> = Some(host2byte(3, 6));

impl_module!(Chassis);

impl<C: Client<V1>> Chassis<V1, C> {
    pub fn set_stick_overlay(&mut self, mode: StickOverlayMode) -> Result<()> {
        self.client.send_cmd_sync(CHASSIS_TARGET_V1, mode)?;
        Ok(())
    }

    pub fn action_start_move(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        xy_speed: Option<f32>,
        z_speed: Option<f32>,
    ) -> Result<(Move<ActionUpdateHead>, Rx<(ActionUpdateHead, MoveUpdate)>)> {
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
        let (mut action, mut rx) = self.action_start_move(x, y, z, xy_speed, z_speed)?;

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
    ) -> Result<(Position, Rx<PositionPush>, Box<dyn Subscription<V1>>)> {
        let (pos_rx, sub) = self.client.subscribe_period_push::<Position>(freq)?;
        Ok((Position::new(origin), pos_rx, sub))
    }
}
