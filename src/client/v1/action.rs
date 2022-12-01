use std::collections::HashMap;
use std::sync::Mutex;

use crate::{
    client::{v1::Client as V1Client, Client, RawHandler},
    proto::{
        v1::{
            action::{
                ActionConfig, ActionHead, ActionSequence, ActionUpdateHead, V1Action,
                ACTION_UPDATE_HEAD_SIZE,
            },
            Seq, V1,
        },
        ActionState, Deserialize, ProtoAction, Raw,
    },
    util::chan::{unbounded, Rx},
    Error, Result,
};

enum ActionCallbackInput<'a> {
    Update(ActionUpdateHead, &'a [u8]),
    Check,
}

type ActionCallback = Box<dyn FnMut(ActionCallbackInput) -> Result<()>>;

pub struct ActionDispatcher {
    seq: ActionSequence,
    client: V1Client,
    callbacks: Mutex<HashMap<Seq, ActionCallback>>,
}

impl ActionDispatcher {
    pub fn send<A: V1Action>(
        &self,
        cfg: Option<ActionConfig>,
        action: &A,
    ) -> Result<Rx<(ActionUpdateHead, A::Update)>>
    where
        A::Update: 'static,
    {
        let seq = self.seq.next();
        let wrapped = (
            ActionHead {
                id: seq as u8,
                cfg: cfg.unwrap_or_default(),
            },
            action,
        );

        let (mut tx, rx) = unbounded();
        let callback: ActionCallback = Box::new(move |input| {
            match input {
                ActionCallbackInput::Update(head, buf) => {
                    let update = <A::Update as Deserialize<V1>>::de(buf)?;
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
            .lock()
            .map(|mut cbs| cbs.insert(seq, callback))
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

impl RawHandler<V1> for ActionDispatcher {
    fn recv(&self, raw: &Raw<V1>) -> Result<bool> {
        if raw.is_ack {
            return Ok(false);
        }

        let (seq, head): (Seq, ActionUpdateHead) = Deserialize::de(&raw.raw_data[..])?;
        let mut callbacks = self
            .callbacks
            .lock()
            .map_err(|_e| Error::Other("callbacks poisoned".into()))?;

        if let Some(cb) = callbacks.get_mut(&seq) {
            cb(ActionCallbackInput::Update(
                head,
                &raw.raw_data[ACTION_UPDATE_HEAD_SIZE..],
            ))?;
            return Ok(true);
        }

        Ok(false)
    }

    fn gc(&self) -> Result<()> {
        let mut callbacks = self
            .callbacks
            .lock()
            .map_err(|_e| Error::Other("callbacks poisoned".into()))?;

        callbacks.retain(|_k, v| v(ActionCallbackInput::Check).is_ok());

        Ok(())
    }
}
