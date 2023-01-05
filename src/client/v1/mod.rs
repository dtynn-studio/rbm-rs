use std::sync::Arc;

use super::{
    ActionDispatcher as ActionDispatcherTrait, Client as ClientTrait,
    Connection as ConnectionTrait, RawHandler, Subscriber as SubscriberTrait,
    Subscription as SubscriptionTrait, TransportRx, TransportRxCloser, TransportTx,
};
use crate::{
    proto::{
        v1::{action::ActionUpdateHead, Receiver, Sender, V1},
        Codec, ProtoAction, ProtoCommand, ProtoPush, ProtoSubscribe,
    },
    util::chan::{Rx, Tx},
    Result,
};

mod action;
mod conn;
mod subscribe;

use action::*;
pub use conn::*;
use subscribe::*;

pub struct Client {
    conn: Arc<Connection>,
    action_dispatcher: ActionDispatcher,
    subscriber: Subscriber,
}

impl Client {
    pub fn new(
        tx: Box<dyn TransportTx>,
        rxs: Vec<Box<dyn TransportRx>>,
        closers: Vec<Box<dyn TransportRxCloser>>,
        host: Sender,
        target: Receiver,
    ) -> Result<Self> {
        let conn = Connection::new(tx, rxs, closers, host, target).map(Arc::new)?;
        let action_dispatcher = ActionDispatcher::new(conn.clone())?;
        let subscriber = Subscriber::new(conn.clone())?;
        Ok(Self {
            conn,
            action_dispatcher,
            subscriber,
        })
    }
}

impl ConnectionTrait<V1> for Client {
    fn send_cmd<CMD: ProtoCommand<V1>>(
        &self,
        receiver: Option<Receiver>,
        cmd: CMD,
        need_ack: bool,
    ) -> Result<Option<CMD::Resp>>
    where
        CMD::Resp: Send + 'static,
    {
        self.conn.send_cmd(receiver, cmd, need_ack)
    }

    fn register_raw_handler<H: RawHandler<V1> + Send + 'static>(
        &self,
        name: &str,
        hdl: H,
    ) -> Result<()> {
        self.conn.register_raw_handler(name, hdl)
    }

    fn unregister_raw_handler(&self, name: &str) -> Result<bool> {
        self.conn.unregister_raw_handler(name)
    }

    fn host(&self) -> Sender {
        self.conn.host()
    }
}

impl ActionDispatcherTrait<V1> for Client {
    fn send_action<PA: ProtoAction<V1>>(
        &self,
        cfg: Option<<V1 as Codec>::ActionConfig>,
        action: &mut PA,
    ) -> Result<Rx<(ActionUpdateHead, PA::Update)>> {
        self.action_dispatcher.send(cfg, action)
    }
}

impl SubscriberTrait<V1> for Client {
    fn subscribe_period_push<PS: ProtoSubscribe<V1>>(
        &self,
        cfg: Option<<V1 as Codec>::SubscribeConfig>,
    ) -> Result<(Rx<PS::Push>, Box<dyn SubscriptionTrait<V1>>)> {
        self.subscriber.subscribe_period_push::<PS>(cfg)
    }

    fn subscribe_event<P: ProtoPush<V1>>(
        &self,
        tx: Tx<P>,
    ) -> Result<Box<dyn SubscriptionTrait<V1>>> {
        self.subscriber.subscribe_event::<P>(tx)
    }
}

impl ClientTrait<V1> for Client {}
