use super::{impl_module, impl_v1_subscribe_meth_simple};
use crate::{client::Client, proto::v1::V1};

pub mod proto;
use proto::sub::Battery;

impl_module!(EPBattery);

impl<C: Client<V1>> EPBattery<V1, C> {
    impl_v1_subscribe_meth_simple!(Battery);
}
