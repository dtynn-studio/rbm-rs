use super::{impl_module, impl_v1_subscribe_meth_simple};
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
};

pub mod proto;
use proto::sub::Battery;

pub const V1_HOST: Option<Receiver> = Some(host2byte(11, 0));

impl_module!(EPBattery);

impl<C: Client<V1>> EPBattery<V1, C> {
    impl_v1_subscribe_meth_simple!(Battery);
}
