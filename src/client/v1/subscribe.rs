use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing::{error, trace};

use crate::proto::{Deserialize, ProtoSubscribe};
use crate::{
    client::{
        v1::Connection as V1Connection, Connection, RawHandler, Subscription as SubscriptionTrait,
    },
    proto::{
        v1::{
            subscribe::{
                PushPeriodMsg, SubConfig, SubMsg, SubscribeSequence, UnsubMsg,
                PUSH_PERIOD_MSG_IDENT,
            },
            Ident, V1,
        },
        ProtoPush, Raw,
    },
    util::{
        chan::{unbounded, Rx, Tx},
        host2byte,
    },
    Error, Result,
};

const HANDLER_NAME: &str = "v1::Subscriber";
const CMD_RECEIVER: u8 = host2byte(9, 0);

enum Subscription {
    PeriodPush {
        msg_id: u8,
        handlers: Arc<SubscribeHandlers>,
        conn: Arc<V1Connection>,
    },

    Event {
        ident: Ident,
        handlers: Arc<SubscribeHandlers>,
    },
}

impl SubscriptionTrait<V1> for Subscription {
    fn unsub(&mut self) -> Result<()> {
        match self {
            Subscription::PeriodPush {
                msg_id,
                handlers,
                conn,
            } => {
                handlers
                    .handlers
                    .lock()
                    .map_err(|_e| Error::Other("handlers poisoned".into()))?
                    .0
                    .remove(msg_id);

                let unsub_msg = UnsubMsg {
                    node_id: conn.host(),
                    msg_id: *msg_id,
                    ..Default::default()
                };

                let _ = conn.send_cmd(Some(CMD_RECEIVER), unsub_msg, false)?;
            }

            Subscription::Event { ident, handlers } => {
                handlers
                    .handlers
                    .lock()
                    .map_err(|_e| Error::Other("handlers poisoned".into()))?
                    .1
                    .remove(ident);
            }
        }

        Ok(())
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Err(e) = self.unsub() {
            error!("unsub failed: {:?}", e);
        }
    }
}

pub struct Subscriber {
    seq: SubscribeSequence,
    handlers: Arc<SubscribeHandlers>,
    conn: Arc<V1Connection>,
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        let _ = self.conn.unregister_raw_handler(HANDLER_NAME);
    }
}

impl Subscriber {
    pub fn new(conn: Arc<V1Connection>) -> Result<Self> {
        let handlers: Arc<SubscribeHandlers> = Default::default();

        conn.register_raw_handler(HANDLER_NAME, handlers.clone())?;

        Ok(Subscriber {
            seq: Default::default(),
            handlers,
            conn,
        })
    }

    pub fn subscribe_period_push<S: ProtoSubscribe<V1>>(
        &self,
        cfg: Option<SubConfig>,
    ) -> Result<(Rx<S::Push>, Box<dyn SubscriptionTrait<V1>>)> {
        let sid = S::SID;
        let msg_id = self.seq.next();
        trace!(%msg_id, %sid, "sub msg");

        let sub_msg = SubMsg::single(self.conn.host(), msg_id, cfg.unwrap_or_default(), sid);

        let (mut tx, rx) = unbounded();
        let hdl: SubscribeHandler = Box::new(move |input| {
            match input {
                SubscribeHandlerInput::Data(data) => {
                    let push = <S::Push as Deserialize<V1>>::de(data)?;
                    tx.send(push)
                        .map_err(|_e| Error::Other("push chan broken".into()))?;
                }

                SubscribeHandlerInput::Check => {
                    if tx.is_closed() {
                        return Err(Error::Other("push chan closed".into()));
                    }
                }
            };

            Ok(())
        });

        self.handlers
            .handlers
            .lock()
            .map(|mut cbs| cbs.0.insert(msg_id, hdl))
            .map_err(|_e| Error::Other("handlers poisoned".into()))?;

        let resp = self
            .conn
            .send_cmd(Some(CMD_RECEIVER), sub_msg, true)?
            .ok_or_else(|| Error::Other("response required for sub msg".into()))?;

        if !resp.ret.is_ok() {
            return Err(Error::Other(
                format!("sub failed with returned info: {:?}", resp).into(),
            ));
        }

        Ok((
            rx,
            Box::new(Subscription::PeriodPush {
                msg_id,
                conn: self.conn.clone(),
                handlers: self.handlers.clone(),
            }),
        ))
    }

    pub fn subscribe_event<P: ProtoPush<V1>>(
        &self,
        mut tx: Tx<P>,
    ) -> Result<Box<dyn SubscriptionTrait<V1>>>
    where
        P: ProtoPush<V1> + Send + 'static,
    {
        let hdl: SubscribeHandler = Box::new(move |input| {
            match input {
                SubscribeHandlerInput::Data(data) => {
                    let event = <P as Deserialize<V1>>::de(data)?;
                    tx.send(event)
                        .map_err(|_e| Error::Other("push chan broken".into()))?;
                }

                SubscribeHandlerInput::Check => {
                    if tx.is_closed() {
                        return Err(Error::Other("push chan closed".into()));
                    }
                }
            };

            Ok(())
        });

        self.handlers
            .handlers
            .lock()
            .map(|mut cbs| cbs.1.insert(P::IDENT, hdl))
            .map_err(|_e| Error::Other("handlers poisoned".into()))?;

        Ok(Box::new(Subscription::Event {
            ident: P::IDENT,
            handlers: self.handlers.clone(),
        }))
    }
}

enum SubscribeHandlerInput<'a> {
    Data(&'a [u8]),
    Check,
}

type SubscribeHandler = Box<dyn FnMut(SubscribeHandlerInput) -> Result<()> + Send>;

#[derive(Default)]
struct SubscribeHandlers {
    handlers: Mutex<(
        HashMap<u8, SubscribeHandler>,    // period push msg handlers
        HashMap<Ident, SubscribeHandler>, // event msg handlers
    )>,
}

impl RawHandler<V1> for Arc<SubscribeHandlers> {
    fn recv(&self, raw: &Raw<V1>) -> Result<bool> {
        if raw.is_ack {
            return Ok(false);
        }

        let mut handlers = self
            .handlers
            .lock()
            .map_err(|_e| Error::Other("subscribe periods handlers poisoned".into()))?;

        if raw.id == PUSH_PERIOD_MSG_IDENT {
            let period_msg = PushPeriodMsg::de(&raw.raw_data)?;

            if let Some(hdl) = handlers.0.get_mut(&period_msg.msg_id) {
                hdl(SubscribeHandlerInput::Data(&period_msg.data))?;
                return Ok(true);
            }
        } else if let Some(hdl) = handlers.1.get_mut(&raw.id) {
            hdl(SubscribeHandlerInput::Data(&raw.raw_data))?;
            return Ok(true);
        }

        Ok(false)
    }

    fn gc(&self) -> Result<()> {
        let mut handlers = self
            .handlers
            .lock()
            .map_err(|_e| Error::Other("subscribe periods handlers poisoned".into()))?;

        handlers
            .0
            .retain(|_k, v| v(SubscribeHandlerInput::Check).is_ok());

        handlers
            .1
            .retain(|_k, v| v(SubscribeHandlerInput::Check).is_ok());

        Ok(())
    }
}
