use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::proto::Deserialize;
use crate::{
    client::RawHandler,
    proto::{
        v1::{
            subscribe::{PushPeriodMsg, PUSH_PERIOD_MSG_IDENT},
            Ident, V1,
        },
        ProtoSubscribe, Raw,
    },
    Error, Result,
};

const HANDLER_NAME: &str = "v1::Subscriber";

pub struct Subscriber {}

impl Subscriber {
    pub fn subscribe_period_push<S: ProtoSubscribe<V1>>(&self) {
        unimplemented!()
    }

    pub fn subscribe_event(&self) {
        unimplemented!()
    }
}

enum SubscribeHandlerInput<'a> {
    Data(&'a [u8]),
    Check,
}

type SubscribeHandler = Box<dyn FnMut(SubscribeHandlerInput) -> Result<()> + Send>;

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
