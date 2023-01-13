use super::{impl_module, impl_v1_subscribe_meth_simple, V1ActionReturn};
use crate::{
    client::Client,
    proto::v1::{action::ActionUpdateHead, Receiver, V1},
    util::host2byte,
    Result,
};

pub mod proto;
use proto::{
    action::{Move, MoveMode},
    sub::Position,
};

pub const V1_HOST: Option<Receiver> = Some(host2byte(27, 2));

impl_module!(RoboticArm);

impl<C: Client<V1>> RoboticArm<V1, C> {
    pub fn action_move_to(
        &mut self,
        x: i32,
        y: i32,
    ) -> Result<V1ActionReturn<Move<ActionUpdateHead>>> {
        let mut action = Move::new(MoveMode::Abs, x, y);
        let rx = self.client.send_action(None, &mut action)?;
        Ok((action, rx))
    }

    pub fn action_move(
        &mut self,
        x: i32,
        y: i32,
    ) -> Result<V1ActionReturn<Move<ActionUpdateHead>>> {
        let mut action = Move::new(MoveMode::Rel, x, y);
        let rx = self.client.send_action(None, &mut action)?;
        Ok((action, rx))
    }

    pub fn action_recenter(&mut self) -> Result<V1ActionReturn<Move<ActionUpdateHead>>> {
        self.action_move_to(0, 0)
    }

    impl_v1_subscribe_meth_simple!(Position);
}
