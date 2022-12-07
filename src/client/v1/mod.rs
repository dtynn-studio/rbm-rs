use std::collections::{hash_map::Entry, HashMap};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use tracing::{debug, trace, warn};

use super::{RawHandler, TransportRx, TransportTx};
use crate::{
    proto::{v1, Codec, Deserialize, ProtoCommand, Raw},
    Error, Result,
};

mod action;
pub use action::*;

type CmdCallback = Box<dyn FnOnce(Result<&Raw<v1::V1>>) + Send + 'static>;

enum Event {
    Cmd {
        id: v1::Ident,
        seq: v1::Seq,
        data: Vec<u8>,
        callback: Option<CmdCallback>,
    },

    RegisterRawHandler {
        name: String,
        hdl: Box<dyn RawHandler<v1::V1> + Send + 'static>,
        resp_tx: Sender<Result<()>>,
    },

    UnregisterRawHandler {
        name: String,
        resp_tx: Sender<bool>,
    },
}

pub struct Client {
    host: v1::Sender,
    target: v1::Receiver,
    cmd_seq: v1::CmdSequence,
    event_tx: Sender<Event>,
    done_tx: Option<Sender<()>>,
    join: Option<thread::JoinHandle<()>>,
}

impl Drop for Client {
    fn drop(&mut self) {
        drop(self.done_tx.take());
        if let Some(join) = self.join.take() {
            debug!("wait for dispatch threads to be stopped");
            let _ = join.join();
        };
    }
}

impl super::Client<v1::V1> for Client {
    fn new(
        tx: Box<dyn super::TransportTx>,
        rxs: Vec<Box<dyn super::TransportRx>>,
        host: <v1::V1 as Codec>::Sender,
        target: <v1::V1 as Codec>::Receiver,
    ) -> Result<Self> {
        let (event_tx, event_rx) = unbounded();
        let (done_tx, done_rx) = bounded(0);

        let join = thread::spawn(move || {
            start_client_inner(tx, rxs, event_rx, done_rx);
        });

        Ok(Self {
            host,
            target,
            cmd_seq: Default::default(),
            event_tx,
            done_tx: Some(done_tx),
            join: Some(join),
        })
    }

    fn send_cmd<CMD: ProtoCommand<v1::V1>>(
        &self,
        receiver: Option<<v1::V1 as Codec>::Receiver>,
        cmd: CMD,
        need_ack: bool,
    ) -> Result<Option<CMD::Resp>>
    where
        CMD::Resp: Send + 'static,
    {
        let data = v1::V1::pack_msg(
            self.host,
            receiver.unwrap_or(self.target),
            0,
            &cmd,
            need_ack,
        )?;

        let seq = self.cmd_seq.next();

        if !need_ack {
            return Ok(None);
        }

        let mut callback: Option<CmdCallback> = None;

        let resp_rx = if need_ack {
            let (resp_tx, resp_rx) = bounded(1);

            callback = Some(Box::new(move |raw_res: Result<&Raw<v1::V1>>| {
                let res =
                    raw_res.and_then(|raw| <CMD::Resp as Deserialize<v1::V1>>::de(&raw.raw_data));
                let _ = resp_tx.send(res);
            }));

            Some(resp_rx)
        } else {
            None
        };

        self.event_tx
            .send(Event::Cmd {
                id: CMD::IDENT,
                seq,
                data,
                callback,
            })
            .map_err(|_e| Error::Other("client task chan broken".into()))?;

        match resp_rx {
            Some(rx) => {
                let res = rx
                    .recv()
                    .map_err(|_e| Error::Other("resp chan broken".into()))?;
                res.map(Some)
            }

            None => Ok(None),
        }
    }

    fn register_raw_handler<H: RawHandler<v1::V1> + Send + 'static>(
        &self,
        name: &str,
        hdl: H,
    ) -> Result<()> {
        let (resp_tx, resp_rx) = bounded(1);
        self.event_tx
            .send(Event::RegisterRawHandler {
                name: name.to_owned(),
                hdl: Box::new(hdl),
                resp_tx,
            })
            .map_err(|_e| Error::Other("client event chan broken".into()))?;

        resp_rx
            .recv()
            .map_err(|_e| Error::Other("register response chan broken".into()))?
    }

    fn unregister_raw_handler<H: RawHandler<v1::V1>>(&self, name: &str) -> Result<bool> {
        let (resp_tx, resp_rx) = bounded(1);
        self.event_tx
            .send(Event::UnregisterRawHandler {
                name: name.to_owned(),
                resp_tx,
            })
            .map_err(|_e| Error::Other("client event chan broken".into()))?;

        resp_rx
            .recv()
            .map_err(|_e| Error::Other("unregister response chan broken".into()))
            .map(Ok)?
    }
}

fn start_client_inner(
    trans_tx: Box<dyn TransportTx>,
    trans_rxs: Vec<Box<dyn TransportRx>>,
    event_rx: Receiver<Event>,
    done: Receiver<()>,
) {
    let (recv_done_tx, recv_done_rx) = bounded::<()>(0);
    let (recv_raw_tx, recv_raw_rx) = unbounded();

    thread::scope(|s| {
        s.spawn(|| {
            debug!("client event loop start");
            if let Err(_e) =
                handle_client_event(done, trans_tx, event_rx, recv_raw_rx, recv_done_rx)
            {
                // TODO: logging
            }
            debug!("client event loop stop");
        });

        // transport recv thread
        for trans_rx in trans_rxs {
            let done_tx = recv_done_tx.clone();
            let raw_tx = recv_raw_tx.clone();
            s.spawn(|| {
                debug!("client recv loop start");
                if let Err(_e) = handle_client_recv(done_tx, trans_rx, raw_tx) {
                    // TODO: logging
                }
                debug!("client recv loop stop");
            });
        }

        drop(recv_done_tx);
        drop(recv_raw_tx);
    });
}

fn handle_client_event(
    done: Receiver<()>,
    mut trans: Box<dyn TransportTx>,
    event_rx: Receiver<Event>,
    raw_rx: Receiver<Raw<v1::V1>>,
    recv_loop_done: Receiver<()>,
) -> Result<()> {
    let mut callbacks: HashMap<(v1::Ident, v1::Seq), (Instant, Option<CmdCallback>)> =
        HashMap::new();
    let mut raw_handlers: HashMap<String, Box<dyn RawHandler<v1::V1> + 'static>> = HashMap::new();
    let mut sent = 0;
    let mut received = 0;
    loop {
        debug!(sent, received, "waiting for client events");
        select! {
            recv(done) -> _ => {
                return Ok(());
            }

            recv(recv_loop_done) -> _ => {
                return Err(Error::Other("recv loop broke unexpectedly".into()));
            }

            recv(event_rx) -> event_res => {
                let event = event_res.map_err(|_| Error::Other("msg chan broken".into()))?;
                match event {
                    Event::Cmd { id, seq, data, callback } => {
                        trans.send(&data)?;

                        callbacks.insert((id, seq), (Instant::now(), callback));

                        sent += 1;
                    },

                    Event::RegisterRawHandler { name, hdl, resp_tx } => {
                        let res = match raw_handlers.entry(name) {
                            Entry::Vacant(v) => {
                                v.insert(hdl);
                                Ok(())
                            },

                            Entry::Occupied(o) => {
                                Err(Error::Other(format!("raw handler {} exists", o.key()).into()))
                            }
                        };

                        if resp_tx.send(res).is_err() {
                            warn!("register response chan broken");
                        };
                    },

                    Event::UnregisterRawHandler { name, resp_tx } => {
                        if resp_tx.send(raw_handlers.remove(name.as_str()).is_some()).is_err() {
                            warn!("unregister response chan broken");
                        };
                    },
                }
            }

            recv(raw_rx) -> raw_res => {
                let raw = raw_res.map_err(|_| Error::Other("raw response chan broken".into()))?;
                received += 1;

                trace!(
                    sender = raw.sender,
                    receiver = raw.receiver,
                    is_ack = raw.is_ack,
                    id = ?raw.id,
                    seq = raw.seq,
                    size = raw.raw_data.len(),
                    "raw data recv"
                );

                if let Some((t_send, maybe_cb)) = callbacks.remove(&(raw.id, raw.seq)) {
                    trace!(latency = ?t_send.elapsed(), "cmd responsed");
                    if let Some(cb) = maybe_cb {
                        cb(Ok(&raw));
                    }
                };

                for (name, hdl) in raw_handlers.iter() {
                    match hdl.recv(&raw) {
                        Ok(handled) => {
                            if handled {
                                debug!(name, "raw handled");
                            }
                        },

                        Err(e) => {
                            warn!(name, "raw handled failed: {:?}", e);
                        }
                    }
                }
            }

            // clenup
            default(Duration::from_secs(300)) => {
                for hdl in raw_handlers.values() {
                    if let Err(_e) = hdl.gc() {
                        // TODO: logging
                    };
                }
            }
        }
    }
}

fn handle_client_recv(
    loop_done: Sender<()>,
    mut recv_trans: Box<dyn TransportRx>,
    raw_tx: Sender<Raw<v1::V1>>,
) -> Result<()> {
    let _done = loop_done;
    let mut buf = [0u8; 2048];
    loop {
        debug!("waiting for incoming msg");
        let read = recv_trans.recv(&mut buf[..])?;
        if read == 0 {
            return Ok(());
        }

        match v1::V1::unpack_raw(&buf[..read]) {
            Ok((raw, _consumed)) => {
                raw_tx
                    .send(raw)
                    .map_err(|_e| Error::Other("raw chan broken".into()))?;
            }

            Err(_e) => {
                // TODO:
                // 1) logging
                // 2) handle as an incompleted buffer
            }
        }
    }
}
