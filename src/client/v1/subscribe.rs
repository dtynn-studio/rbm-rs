use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing::warn;

use crate::proto::Deserialize;
use crate::{
    client::{v1::Client as V1Client, Client, RawHandler},
    proto::{
        v1::{
            subscribe::{
                PushPeriodMsg, PushPeriodSubject, SubConfig, SubMsg, SubscribeSequence, UnsubMsg,
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

pub struct PushSubscription<S: PushPeriodSubject> {
    pub rx: Rx<S>,
    msg_id: u8,
    handlers: Arc<SubscribeHandlers>,
    client: Arc<V1Client>,
}

pub struct EventSubscription {
    ident: Ident,
    handlers: Arc<SubscribeHandlers>,
}

impl EventSubscription {
    fn unsub(&mut self) -> Result<()> {
        self.handlers
            .handlers
            .lock()
            .map_err(|_e| Error::Other("handlers poisoned".into()))?
            .1
            .remove(&self.ident);

        Ok(())
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        if let Err(e) = self.unsub() {
            warn!(ident = ?self.ident, "event unsub failed: {:?}", e);
        }
    }
}

impl<S: PushPeriodSubject> Drop for PushSubscription<S> {
    fn drop(&mut self) {
        if let Err(e) = self.unsub() {
            warn!(
                uid = S::UID,
                msg_id = self.msg_id,
                "push unsub failed: {:?}",
                e
            );
        }
    }
}

impl<S: PushPeriodSubject> PushSubscription<S> {
    fn unsub(&mut self) -> Result<()> {
        self.handlers
            .handlers
            .lock()
            .map_err(|_e| Error::Other("handlers poisoned".into()))?
            .0
            .remove(&self.msg_id);

        let unsub_msg = UnsubMsg {
            node_id: self.client.host(),
            msg_id: self.msg_id,
            ..Default::default()
        };

        let _ = self
            .client
            .send_cmd(Some(CMD_RECEIVER), unsub_msg, true)?
            .ok_or_else(|| Error::Other("response required for sub msg".into()))?;

        Ok(())
    }
}

pub struct Subscriber {
    seq: SubscribeSequence,
    handlers: Arc<SubscribeHandlers>,
    client: Arc<V1Client>,
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        let _ = self.client.unregister_raw_handler(HANDLER_NAME);
    }
}

impl Subscriber {
    pub fn new(client: Arc<V1Client>) -> Result<Self> {
        let handlers: Arc<SubscribeHandlers> = Default::default();

        client.register_raw_handler(HANDLER_NAME, handlers.clone())?;

        Ok(Subscriber {
            seq: Default::default(),
            handlers,
            client,
        })
    }

    pub fn subscribe_period_push<S: PushPeriodSubject>(
        &self,
        cfg: Option<SubConfig>,
    ) -> Result<PushSubscription<S>>
    where
        S: PushPeriodSubject + Send + 'static,
    {
        let msg_id = self.seq.next();
        let sub_msg = SubMsg::single(self.client.host(), msg_id, cfg.unwrap_or_default(), S::UID);

        let (mut tx, rx) = unbounded();
        let hdl: SubscribeHandler = Box::new(move |input| {
            match input {
                SubscribeHandlerInput::Data(data) => {
                    let push = <S as Deserialize<V1>>::de(data)?;
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
            .client
            .send_cmd(Some(CMD_RECEIVER), sub_msg, true)?
            .ok_or_else(|| Error::Other("response required for sub msg".into()))?;

        if !resp.ret.is_ok() {
            return Err(Error::Other(
                format!("sub failed with returned info: {:?}", resp).into(),
            ));
        }

        Ok(PushSubscription {
            rx,
            msg_id,
            client: self.client.clone(),
            handlers: self.handlers.clone(),
        })
    }

    pub fn subscribe_event<P: ProtoPush<V1>>(&self, mut tx: Tx<P>) -> Result<EventSubscription>
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

        Ok(EventSubscription {
            ident: P::IDENT,
            handlers: self.handlers.clone(),
        })
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
