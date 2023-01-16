use super::{impl_module, impl_v1_subscribe_meth_simple};
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::{host2byte, unit_convertor},
    Result,
};

pub mod proto;
use proto::{
    cmd::{Control, ControlOp},
    sub::Status,
};

pub const V1_HOST: Option<Receiver> = Some(proto::cmd::HOST_ID);

impl_module!(Gripper);

impl<C: Client<V1>> Gripper<V1, C> {
    pub fn open(&mut self, power: Option<f32>) -> Result<()> {
        let pow = unit_convertor::GRIPPER_POWER_CONVERTOR.val2proto(power.unwrap_or(50.0))?;
        let cmd = Control::new(ControlOp::Open, pow);
        self.client.send_cmd_sync(Some(host2byte(3, 6)), cmd)?;

        Ok(())
    }

    pub fn close(&mut self, power: Option<f32>) -> Result<()> {
        let pow = unit_convertor::GRIPPER_POWER_CONVERTOR.val2proto(power.unwrap_or(50.0))?;
        let cmd = Control::new(ControlOp::Close, pow);
        self.client.send_cmd_sync(Some(host2byte(3, 6)), cmd)?;

        Ok(())
    }

    impl_v1_subscribe_meth_simple!(Status);
}
