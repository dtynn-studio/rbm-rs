use super::{impl_module, impl_v1_subscribe_meth_simple, V1ActionReturn};
use crate::{
    client::Client,
    proto::v1::{action::ActionUpdateHead, Receiver, V1},
    util::host2byte,
    Result,
};

pub mod proto;
pub use proto::ServoIndex;
use proto::{
    action::SetAngle,
    cmd::{Control, GetAngle, SetMode},
    sub::ServoState,
};

pub const V1_HOST: Option<Receiver> = Some(host2byte(3, 5));

impl_module!(Servo);

impl<C: Client<V1>> Servo<V1, C> {
    pub fn set_speed(&mut self, idx: ServoIndex, speed: u16) -> Result<()> {
        self.client
            .send_cmd_sync(V1_HOST, SetMode { idx, mode: 1 })?;
        self.client
            .send_cmd_sync(V1_HOST, Control::new(idx, (speed + 49) * 900 / 98))?;
        Ok(())
    }

    pub fn pause(&mut self, idx: ServoIndex) -> Result<()> {
        self.client.send_cmd_sync(V1_HOST, Control::disabled(idx))?;
        Ok(())
    }

    pub fn get_angle(&mut self, idx: ServoIndex) -> Result<i32> {
        self.client
            .send_cmd_sync(V1_HOST, GetAngle::from(idx))
            .map(|resp| resp.angle)
    }

    pub fn action_move_to(
        &mut self,
        idx: ServoIndex,
        angle: i32,
    ) -> Result<V1ActionReturn<SetAngle<ActionUpdateHead>>> {
        let mut action = SetAngle::new(idx, angle);
        let rx = self.client.send_action(None, &mut action)?;
        Ok((action, rx))
    }

    impl_v1_subscribe_meth_simple!(ServoState);
}
