use super::super::{impl_module, impl_v1_subscribe_meth_simple};
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
    Result,
};

pub mod proto;
use proto::{
    cmd::{SensorData, SensorTarget},
    sub::Pinboard,
    SensorIndex, SensorPort,
};

const V1_HOST_NUM: u8 = 22;
pub const V1_HOST: Option<Receiver> = Some(host2byte(V1_HOST_NUM, 0));

impl_module!(Adaptor);

impl<C: Client<V1>> Adaptor<V1, C> {
    pub fn get_sensor_data(&mut self, idx: SensorIndex, port: SensorPort) -> Result<SensorData> {
        self.client.send_cmd_sync(
            Some(host2byte(V1_HOST_NUM, idx as u8)),
            SensorTarget::from(port),
        )
    }

    impl_v1_subscribe_meth_simple!(Pinboard);
}
