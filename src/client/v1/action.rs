use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    client::{v1::Client as V1Client, Client, RawHandler},
    proto::{
        v1::{
            action::{
                ActionConfig, ActionHead, ActionSequence, ActionUpdateHead, V1Action,
                ACTION_UPDATE_HEAD_SIZE,
            },
            Ident, Seq, V1,
        },
        ActionState, Deserialize, ProtoAction, ProtoPush, Raw,
    },
    util::chan::{unbounded, Rx},
    Error, Result,
};

const HANDLER_NAME: &str = "v1::actions::ActionDispatcher";

enum ActionCallbackInput<'a> {
    Update(ActionUpdateHead, &'a Raw<V1>, usize),
    Check,
}

type ActionCallback = Box<dyn FnMut(ActionCallbackInput) -> Result<()> + Send>;

#[derive(Default)]
struct ActionCallbacks(Mutex<HashMap<(Ident, Seq), ActionCallback>>);

pub struct ActionDispatcher {
    seq: ActionSequence,
    client: Arc<V1Client>,
    callbacks: Arc<ActionCallbacks>,
}

impl Drop for ActionDispatcher {
    fn drop(&mut self) {
        let _ = self.client.unregister_raw_handler(HANDLER_NAME);
    }
}

impl ActionDispatcher {
    pub fn new(client: Arc<V1Client>) -> Result<Self> {
        let callbacks: Arc<ActionCallbacks> = Default::default();

        client.register_raw_handler(HANDLER_NAME, callbacks.clone())?;

        Ok(Self {
            seq: Default::default(),
            client,
            callbacks,
        })
    }

    pub fn send<A: V1Action>(
        &self,
        cfg: Option<ActionConfig>,
        action: &A,
    ) -> Result<Rx<(ActionUpdateHead, A::Update)>>
    where
        A::Update: Send + 'static,
    {
        let seq = self.seq.next();
        let wrapped = (
            ActionHead {
                id: seq as u8,
                cfg: cfg.unwrap_or_default(),
            },
            action,
        );

        let update_ident = <A::Update as ProtoPush<V1>>::IDENT;

        let (mut tx, rx) = unbounded();
        let callback: ActionCallback = Box::new(move |input| {
            match input {
                ActionCallbackInput::Update(head, raw, used) => {
                    let update = <A::Update as Deserialize<V1>>::de(&raw.raw_data[used..])?;
                    tx.send((head, update))
                        .map_err(|_e| Error::Other("update chan broken".into()))?;
                }

                ActionCallbackInput::Check => {
                    if !tx.is_closed() {
                        return Err(Error::Other("update chan closed".into()));
                    }
                }
            };

            Ok(())
        });

        self.callbacks
            .0
            .lock()
            .map(|mut cbs| cbs.insert((update_ident, seq), callback))
            .map_err(|_e| Error::Other("callbacks poisoned".into()))?;

        let cmd = wrapped.pack_cmd()?;
        let resp = self
            .client
            .send_cmd(A::TARGET, cmd, true)?
            .ok_or_else(|| Error::Other("response required but not received".into()))?;

        let state: ActionState = resp.into();
        if !matches!(state, ActionState::Started | ActionState::Succeeded) {
            return Err(Error::Other(
                format!("action responsed: {:?}", state).into(),
            ));
        }

        Ok(rx)
    }
}

impl RawHandler<V1> for Arc<ActionCallbacks> {
    fn recv(&self, raw: &Raw<V1>) -> Result<bool> {
        if raw.is_ack {
            return Ok(false);
        }

        let (seq, head): (Seq, ActionUpdateHead) = Deserialize::de(&raw.raw_data[..])?;
        let mut callbacks = self
            .0
            .lock()
            .map_err(|_e| Error::Other("callbacks poisoned".into()))?;

        if let Some(cb) = callbacks.get_mut(&(raw.id, seq)) {
            cb(ActionCallbackInput::Update(
                head,
                raw,
                ACTION_UPDATE_HEAD_SIZE,
            ))?;
            return Ok(true);
        }

        Ok(false)
    }

    fn gc(&self) -> Result<()> {
        let mut callbacks = self
            .0
            .lock()
            .map_err(|_e| Error::Other("callbacks poisoned".into()))?;

        callbacks.retain(|_k, v| v(ActionCallbackInput::Check).is_ok());

        Ok(())
    }
}
