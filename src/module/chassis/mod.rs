use std::sync::Arc;

use crate::{
    client::Connection,
    proto::{
        v1::{action::ActionUpdateHead, Receiver, V1},
        Codec,
    },
    util::host2byte,
    Result,
};

pub mod proto;
use proto::{action::Move, cmd::StickOverlayMode};

pub struct Chassis<CODEC: Codec, C: Connection<CODEC>> {
    client: Arc<C>,

    _codec: std::marker::PhantomData<CODEC>,
}

const CHASSIS_TARGET_V1: Option<Receiver> = Some(host2byte(3, 6));

impl<C: Connection<V1>> Chassis<V1, C> {
    pub fn set_stick_overlay(&mut self, mode: StickOverlayMode) -> Result<()> {
        self.client.send_cmd_sync(CHASSIS_TARGET_V1, mode)?;
        Ok(())
    }

    pub fn move_to(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        xy_speed: Option<f32>,
        z_speed: Option<f32>,
    ) -> Result<()> {
        let action = Move::<ActionUpdateHead>::new(
            x,
            y,
            z,
            xy_speed.unwrap_or(0.5),
            z_speed.unwrap_or(30.0),
        );
        unimplemented!()
    }
}
