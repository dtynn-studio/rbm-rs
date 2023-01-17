use crate::{
    client::Subscription,
    proto::{
        v1::{action::ActionUpdateHead, V1},
        ProtoAction, ProtoSubscribe,
    },
    util::chan::Rx,
};

pub mod ai;
pub mod armor;
pub mod battery;
pub mod blaster;
pub mod camera;
pub mod chassis;
pub mod common;
pub mod dds;
pub mod gimbal;
pub mod gripper;
pub mod led;
pub mod robotic_arm;
pub mod sensor;
pub mod servo;
pub mod uart;
pub mod vision;

pub(self) mod util;
use util::{impl_module, impl_v1_subscribe_meth_simple};

pub type V1ActionReturn<T> = (T, Rx<(ActionUpdateHead, <T as ProtoAction<V1>>::Update)>);
pub type V1SubscribeReturn<T> = (
    T,
    Rx<<T as ProtoSubscribe<V1>>::Push>,
    Box<dyn Subscription<V1>>,
);
