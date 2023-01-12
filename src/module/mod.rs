use crate::{
    client::Subscription,
    proto::{
        v1::{action::ActionUpdateHead, V1},
        ProtoAction, ProtoSubscribe,
    },
    util::chan::Rx,
};

pub mod blaster;
pub mod camera;
pub mod chassis;
pub mod common;
pub mod dds;
pub mod gimbal;
pub mod led;
pub mod vision;

pub(self) mod util;
use util::{impl_module, impl_v1_subscribe_meth_simple};

pub type V1ActionReturn<T> = (T, Rx<(ActionUpdateHead, <T as ProtoAction<V1>>::Update)>);
pub type V1SubscribeReturn<T> = (
    T,
    Rx<<T as ProtoSubscribe<V1>>::Push>,
    Box<dyn Subscription<V1>>,
);
