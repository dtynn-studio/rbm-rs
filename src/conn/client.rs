#![allow(clippy::type_complexity)]

use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender, TryRecvError};
use tracing::{debug, trace};

use super::transport::Transport;
use crate::{
    proto::{
        action::{Action, ActionCommand, Progress},
        cmd::Command,
        Codec, CodecCtx, Completed, Deserialize, DussMBAck, Event, Message,
    },
    Error, Result,
};

pub struct ActionProgressRx<R, S, E>
where
    R: Debug,
    S: Debug,
    E: Debug,
{
    rx: Receiver<Progress<R, S, E>>,
    _done: Sender<()>,
}

impl<R: Debug, S: Debug, E: Debug> ActionProgressRx<R, S, E> {
    pub fn next(&self) -> Option<Progress<R, S, E>> {
        self.rx.recv().ok()
    }

    pub fn receiver(&self) -> &Receiver<Progress<R, S, E>> {
        &self.rx
    }
}

struct ActionProgressHandler<C: Codec> {
    cmd_id: (C::Ident, C::Seq),
    action_id: (C::Ident, C::Seq),
    resp_hdl: Box<dyn Fn(C::ActionResponse) -> Result<bool> + Send + Sync>,
    evt_hdl: Box<dyn Fn(C::ActionStatus, &[u8]) -> Result<bool> + Send + Sync>,
    done_rx: Receiver<()>,
}

impl<C: Codec> ActionProgressHandler<C> {
    fn is_closed(&self) -> bool {
        matches!(self.done_rx.try_recv(), Err(TryRecvError::Disconnected))
    }

    fn try_send_resp(&self, data: &[u8]) -> Result<bool> {
        let resp = C::ActionResponse::de(data)?;
        (self.resp_hdl)(resp)
    }

    fn try_send_event(&self, status: C::ActionStatus, data: &[u8]) -> Result<bool> {
        (self.evt_hdl)(status, data)
    }
}

pub struct Client<C>
where
    C: Codec,
{
    host: u8,
    target: u8,
    cmd_tx: Sender<((C::Ident, C::Seq), Vec<u8>, Option<Sender<Vec<u8>>>)>,
    action_tx: Sender<(Vec<u8>, Arc<ActionProgressHandler<C>>)>,
    codec: Arc<C>,
    done_tx: Option<Sender<()>>,
    join: Option<thread::JoinHandle<()>>,
}

impl<C> Client<C>
where
    C: Codec + 'static,
{
    pub fn connect<T: Transport + 'static>(
        bind: Option<SocketAddr>,
        dest: SocketAddr,
        host: u8,
        target: u8,
    ) -> Result<Self>
    where
        T: Transport,
    {
        debug!(?bind, ?dest, "connecting");

        let sender_trans = T::connect(bind, dest)?;
        let recv_trans = sender_trans.try_clone()?;

        let codec = Arc::new(C::default());
        let (cmd_tx, cmd_rx) = unbounded();
        let (action_tx, action_rx) = unbounded();
        let (done_tx, done_rx) = bounded(0);

        let join = thread::spawn(move || {
            start_client_inner::<T, C>(sender_trans, recv_trans, cmd_rx, action_rx, done_rx);
        });

        Ok(Self {
            host,
            target,
            cmd_tx,
            action_tx,
            codec,
            done_tx: Some(done_tx),
            join: Some(join),
        })
    }

    pub fn send_cmd<CMD>(
        &self,
        receiver: Option<u8>,
        cmd: CMD,
        need_ack: Option<DussMBAck>,
    ) -> Result<Option<CMD::Response>>
    where
        CMD: Command<Ident = C::Ident>,
    {
        let ctx = C::ctx::<CMD>(self.host, receiver.unwrap_or(self.target), need_ack);

        let no_ret = ctx.need_ack() == DussMBAck::No;
        let cmd_seq = self.codec.next_cmd_seq();
        let data = self.codec.pack_msg(ctx, cmd, cmd_seq)?;
        if no_ret {
            self.cmd_tx
                .send(((CMD::IDENT, cmd_seq), data, None))
                .map_err(|_| Error::Other("sending chan broken".into()))?;
            return Ok(None);
        }

        let (resp_tx, resp_rx) = bounded(1);
        self.cmd_tx
            .send(((CMD::IDENT, cmd_seq), data, Some(resp_tx)))
            .map_err(|_| Error::Other("sending chan broken".into()))?;

        let resp_data = resp_rx
            .recv()
            .map_err(|_| Error::Other("response chan broken".into()))?;

        <CMD as Command>::Response::de(&resp_data[..]).map(Some)
    }

    pub fn send_action<A>(
        &self,
        action: &A,
    ) -> Result<ActionProgressRx<<A::Cmd as Command>::Response, A::Status, A::Event>>
    where
        A: Action<Status = C::ActionStatus> + Sync + Send + 'static,
        A::Cmd: ActionCommand<Ident = C::Ident, Response = C::ActionResponse, Seq = C::Seq>,
        A::Event: Event<Ident = C::Ident> + Send,
    {
        let mut cmd = action.pack_cmd()?;
        let ctx = C::ctx::<A::Cmd>(self.host, A::RECEIVER, None);
        let cmd_seq = self.codec.next_cmd_seq();
        let action_seq = self.codec.next_action_seq();
        cmd.set_action_seq(action_seq);
        let data = self.codec.pack_msg(ctx, cmd, cmd_seq)?;

        let (progres_tx, progres_rx) = unbounded();

        let progres_tx2 = progres_tx.clone();
        let (done_tx, done_rx) = bounded(0);

        let hdl: ActionProgressHandler<C> = ActionProgressHandler {
            cmd_id: (<A::Cmd as Message>::IDENT, cmd_seq),
            action_id: (<A::Event as Event>::IDENT, action_seq),
            resp_hdl: Box::new(move |resp| {
                let completed = resp.is_completed();
                progres_tx
                    .send(Progress::Response(resp))
                    .map(|_| completed)
                    .or(Ok(true))
            }),
            evt_hdl: Box::new(move |status, data| {
                let evt = A::Event::de(data)?;
                let completed = status.is_completed();
                progres_tx2
                    .send(Progress::Event(status, evt))
                    .map(|_| completed)
                    .or(Ok(true))
            }),
            done_rx,
        };

        self.action_tx
            .send((data, Arc::new(hdl)))
            .map_err(|_| Error::Other("sending chan broken".into()))?;

        Ok(ActionProgressRx {
            rx: progres_rx,
            _done: done_tx,
        })
    }
}

impl<C> Drop for Client<C>
where
    C: Codec,
{
    fn drop(&mut self) {
        drop(self.done_tx.take());
        if let Some(join) = self.join.take() {
            debug!("wait for dispatch threads to be stopped");
            let _ = join.join();
        };
    }
}

fn start_client_inner<T, C>(
    mut sender_trans: T,
    mut recv_trans: T,
    cmd_rx: Receiver<((C::Ident, C::Seq), Vec<u8>, Option<Sender<Vec<u8>>>)>,
    action_rx: Receiver<(Vec<u8>, Arc<ActionProgressHandler<C>>)>,
    done: Receiver<()>,
) where
    T: Transport,
    C: Codec,
{
    let (recv_done_tx, recv_done_rx) = bounded::<()>(0);
    let (recv_raw_tx, recv_raw_rx) = unbounded();

    thread::scope(|s| {
        s.spawn(|| {
            debug!("client dispatch loop start");
            if let Err(_e) = start_client_dispatch::<T, C>(
                done,
                &mut sender_trans,
                cmd_rx,
                action_rx,
                recv_raw_rx,
                recv_done_rx,
            ) {
                // TODO: logging
            }
            sender_trans.shutdown();
            debug!("client dispatch loop stop");
        });

        // transport recv thread
        s.spawn(|| {
            debug!("client recv loop start");
            if let Err(_e) = start_client_recv::<T, C>(recv_done_tx, &mut recv_trans, recv_raw_tx) {
                // TODO: logging
            }
            debug!("client recv loop stop");
        });
    });
}

fn start_client_dispatch<T, C>(
    done: Receiver<()>,
    trans: &mut T,
    cmd_rx: Receiver<((C::Ident, C::Seq), Vec<u8>, Option<Sender<Vec<u8>>>)>,
    action_rx: Receiver<(Vec<u8>, Arc<ActionProgressHandler<C>>)>,
    raw_tx: Receiver<((C::Ident, C::Seq), C::Ctx, Vec<u8>)>,
    recv_loop_done: Receiver<()>,
) -> Result<()>
where
    T: Transport,
    C: Codec,
{
    let mut pending_cmds = HashMap::new();
    let mut pending_action_resp_hdls = HashMap::new();
    let mut pending_action_event_hdls = HashMap::new();
    let mut sent_cmd = 0;
    let mut sent_action = 0;
    let mut recv_resp = 0;
    let mut recv_action_event = 0;
    'DISPATCH_LOOP: loop {
        debug!(
            sent_cmd,
            sent_action, recv_resp, recv_action_event, "waiting for client events"
        );
        select! {
            recv(done) -> _ => {
                return Ok(());
            }

            recv(recv_loop_done) -> _ => {
                return Err(Error::Other("recv loop broke unexpectedly".into()));
            }

            recv(cmd_rx) -> msg_res => {
                let (msg_id, data, maybe_resp) = msg_res.map_err(|_| Error::Other("msg chan broken".into()))?;
                trace!(?data, "cmd data");
                trans.send(&data[..])?;
                debug!(?msg_id, size = data.len(), pending = maybe_resp.is_some(), "cmd data sent");
                if let Some(resp_tx) = maybe_resp {
                    pending_cmds.insert(msg_id, resp_tx);
                }

                sent_cmd += 1;
            }

            recv(action_rx) -> action_res => {
                let (data, hdl) = action_res.map_err(|_| Error::Other("action chan broken".into()))?;
                trace!(?data, "action data");
                trans.send(&data[..])?;
                let (cmd_id, action_id) = (hdl.cmd_id, hdl.action_id);
                debug!(?cmd_id, ?action_id, size = data.len(), "action data sent");
                pending_action_resp_hdls.insert(cmd_id, hdl.clone());
                pending_action_event_hdls.insert(action_id, hdl);

                sent_action += 1;
            }

            recv(raw_tx) -> raw_res => {
                let (msg_id, _msg_ctx, raw_data) = raw_res.map_err(|_| Error::Other("raw response chan broken".into()))?;
                trace!(?raw_data, "recv raw data");
                if let Some(tx) = pending_cmds.remove(&msg_id) {
                    if let Err(_e) = tx.send(raw_data) {
                        // TODO: logging
                    }

                    recv_resp += 1;
                    continue 'DISPATCH_LOOP;
                }

                if let Some(hdl) = pending_action_resp_hdls.get(&msg_id) {
                    if let Err(_e) = hdl.try_send_resp(&raw_data) {
                        // TODO: logging

                    }

                    recv_resp += 1;
                    continue 'DISPATCH_LOOP;
                }

                // try action event
                match C::unpack_action_status(&raw_data) {
                    Ok((action_seq, status, used)) => {
                        if let Some(hdl) = pending_action_event_hdls.get(&(msg_id.0, action_seq)) {
                            if let Err(_e) = hdl.try_send_event(status, &raw_data[used..]) {
                                // TODO: logging
                            }
                        }

                        recv_action_event += 1;
                        continue 'DISPATCH_LOOP;
                    },

                    Err(_e) => {
                        // TODO: logging
                    }
                }


                // TODO: dispatch subsribed events
            }

            // clenup
            default(Duration::from_secs(300)) => {
                pending_action_resp_hdls.retain(|_, hdl: &mut Arc<ActionProgressHandler<C>>| !hdl.as_ref().is_closed());
                pending_action_event_hdls.retain(|_, hdl: &mut Arc<ActionProgressHandler<C>>| !hdl.as_ref().is_closed());
            }
        }
    }
}

fn start_client_recv<T, C>(
    loop_done: Sender<()>,
    recv_trans: &mut T,
    raw_tx: Sender<((C::Ident, C::Seq), C::Ctx, Vec<u8>)>,
) -> Result<()>
where
    T: Transport,
    C: Codec,
{
    let _done = loop_done;
    let mut buf = [0u8; 2048];
    loop {
        debug!("waiting for incoming msg");
        let read = recv_trans.recv(&mut buf[..])?;
        if read == 0 {
            return Ok(());
        }

        match C::unpack_raw(&buf[..read]) {
            Ok((msg_id, recv_ctx, data, consumed)) => {
                debug!(
                    ?msg_id,
                    ?recv_ctx,
                    consumed,
                    size = data.len(),
                    "raw msg unpacked"
                );
                raw_tx
                    .send((msg_id, recv_ctx, data.into()))
                    .map_err(|_e| Error::Other("raw chan broken".into()))?;
            }

            Err(_e) => {
                // TODO: logging
            }
        }
    }
}
