use super::{impl_module, impl_v1_subscribe_meth_simple, V1ActionReturn};
use crate::{
    client::Client,
    proto::v1::{action::ActionUpdateHead, Receiver, V1},
    util::{host2byte, unit_convertor},
    Result,
};

pub mod proto;
use proto::{
    action::{Move, Recenter},
    cmd::{CtrlCode, SetCtrlSpeed},
    sub::Position,
};

pub const V1_HOST: Option<Receiver> = Some(host2byte(4, 0));

impl_module!(Gimbal);

impl<C: Client<V1>> Gimbal<V1, C> {
    pub fn suspend(&mut self) -> Result<()> {
        self.client.send_cmd_sync(V1_HOST, CtrlCode::Suspend)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.client.send_cmd_sync(V1_HOST, CtrlCode::Resume)?;
        Ok(())
    }

    pub fn set_speed(&mut self, pitch: f32, yaw: f32) -> Result<()> {
        let cmd = SetCtrlSpeed {
            yaw: unit_convertor::GIMBAL_YAW_SPEED_SET_CONVERTOR.val2proto(yaw)?,
            pitch: unit_convertor::GIMBAL_PITCH_SPEED_SET_CONVERTOR.val2proto(pitch)?,
            ..Default::default()
        };

        self.client.send_cmd_sync(V1_HOST, cmd)?;
        Ok(())
    }

    pub fn action_recenter(
        &mut self,
        yaw_speed: f32,
        pitch_speed: f32,
    ) -> Result<V1ActionReturn<Recenter<ActionUpdateHead>>> {
        let mut action = Recenter::<ActionUpdateHead>::new(yaw_speed, pitch_speed);
        let rx = self.client.send_action(None, &mut action)?;
        Ok((action, rx))
    }

    pub fn action_move(
        &mut self,
        yaw: f32,
        pitch: f32,
        yaw_speed: f32,
        pitch_speed: f32,
    ) -> Result<V1ActionReturn<Move<ActionUpdateHead>>> {
        let mut action = Move::new_move((yaw, pitch), (yaw_speed, pitch_speed))?;
        let rx = self.client.send_action(None, &mut action)?;
        Ok((action, rx))
    }

    pub fn action_move_to(
        &mut self,
        yaw: f32,
        pitch: f32,
        yaw_speed: f32,
        pitch_speed: f32,
    ) -> Result<V1ActionReturn<Move<ActionUpdateHead>>> {
        let mut action = Move::new_move_to((yaw, pitch), (yaw_speed, pitch_speed))?;
        let rx = self.client.send_action(None, &mut action)?;
        Ok((action, rx))
    }

    impl_v1_subscribe_meth_simple!(Position);
}
