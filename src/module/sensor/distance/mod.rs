use super::super::{impl_module, impl_v1_subscribe_meth_simple};
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::host2byte,
};

pub mod proto;
use proto::sub::Tofs;

pub const V1_HOST: Option<Receiver> = Some(host2byte(18, 1));

impl_module!(Distance);

impl<C: Client<V1>> Distance<V1, C> {
    impl_v1_subscribe_meth_simple!(Tofs);
}
