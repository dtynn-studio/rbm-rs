#![allow(clippy::type_complexity)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};

use super::transport::Transport;
use crate::{
    proto::{Codec, CodecCtx, Deserialize, DussMBAck, Message},
    Error, Result,
};

pub struct Client<C>
where
    C: Codec,
{
    host: u8,
    target: u8,
    msg_tx: Sender<(C::MsgIdent, Vec<u8>, Option<Sender<Vec<u8>>>)>,
    codec: Arc<C>,
    _done_tx: Sender<()>,
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
    ) -> Result<Self> {
        let sender_trans = T::connect(bind, dest)?;
        let recv_trans = sender_trans.try_clone()?;
        let codec = Arc::new(C::default());
        let (msg_tx, msg_rx) = unbounded();
        let (done_tx, done_rx) = bounded(0);

        thread::spawn(move || {
            start_client_inner::<T, C>(sender_trans, recv_trans, msg_rx, done_rx);
        });

        Ok(Self {
            host,
            target,
            msg_tx,
            codec,
            _done_tx: done_tx,
        })
    }

    pub fn send_msg<M>(
        &self,
        receiver: Option<u8>,
        msg: M,
        need_ack: Option<DussMBAck>,
    ) -> Result<Option<M::Response>>
    where
        M: Message<Ident = C::CmdIdent>,
    {
        let ctx = C::ctx::<M>(self.host, receiver.unwrap_or(self.target), need_ack);

        let no_ret = ctx.need_ack() == DussMBAck::No;
        let (msg_id, data) = self.codec.pack_msg(ctx, msg)?;
        if no_ret {
            self.msg_tx
                .send((msg_id, data, None))
                .map_err(|_| Error::Other("sending chan broken".into()))?;
            return Ok(None);
        }

        let (resp_tx, resp_rx) = bounded(1);
        self.msg_tx
            .send((msg_id, data, Some(resp_tx)))
            .map_err(|_| Error::Other("sending chan broken".into()))?;

        let resp_data = resp_rx
            .recv()
            .map_err(|_| Error::Other("response chan broken".into()))?;

        <M as Message>::Response::de(&resp_data[..]).map(Some)
    }
}

fn start_client_inner<T, C>(
    mut sender_trans: T,
    mut recv_trans: T,
    msg_rx: Receiver<(C::MsgIdent, Vec<u8>, Option<Sender<Vec<u8>>>)>,
    done: Receiver<()>,
) where
    T: Transport,
    C: Codec,
{
    let (recv_done_tx, recv_done_rx) = bounded::<()>(0);
    let (recv_raw_tx, recv_raw_rx) = unbounded();

    thread::scope(|s| {
        s.spawn(|| {
            if let Err(_e) = start_client_dispatch::<T, C>(
                done,
                &mut sender_trans,
                msg_rx,
                recv_raw_rx,
                recv_done_rx,
            ) {
                // TODO: logging
            }
        });

        // transport recv thread
        s.spawn(|| {
            if let Err(_e) = start_client_recv::<T, C>(recv_done_tx, &mut recv_trans, recv_raw_tx) {
                // TODO: logging
            }
        });
    });
}

fn start_client_dispatch<T, C>(
    done: Receiver<()>,
    trans: &mut T,
    msg_rx: Receiver<(C::MsgIdent, Vec<u8>, Option<Sender<Vec<u8>>>)>,
    raw_tx: Receiver<(C::MsgIdent, C::Ctx, Vec<u8>)>,
    recv_loop_done: Receiver<()>,
) -> Result<()>
where
    T: Transport,
    C: Codec,
{
    let mut pending = HashMap::new();
    loop {
        select! {
            recv(done) -> _ => {
                trans.shutdown();
                return Ok(());
            }

            recv(recv_loop_done) -> _ => {
                return Err(Error::Other("recv loop broke unexpectedly".into()));
            }

            recv(msg_rx) -> msg_res => {
                let (msg_id, data, maybe_resp) = msg_res.map_err(|_| Error::Other("msg chan broken".into()))?;
                trans.send(&data[..])?;
                if let Some(resp_tx) = maybe_resp {
                    pending.insert(msg_id, resp_tx);
                }
            }

            recv(raw_tx) -> raw_res => {
                let (msg_id, _msg_ctx, raw_data) = raw_res.map_err(|_| Error::Other("raw response chan broken".into()))?;
                if let Some(tx) = pending.remove(&msg_id) {
                    if let Err(_e) = tx.send(raw_data) {
                        // TODO: logging
                    }
                }

                // TODO: dispatch subsribed events
            }
        }
    }
}

fn start_client_recv<T, C>(
    loop_done: Sender<()>,
    recv_trans: &mut T,
    raw_tx: Sender<(C::MsgIdent, C::Ctx, Vec<u8>)>,
) -> Result<()>
where
    T: Transport,
    C: Codec,
{
    let _done = loop_done;
    let mut buf = [0u8; 2048];
    loop {
        let read = recv_trans.recv(&mut buf[..])?;

        match C::unpack_raw(&buf[..read]) {
            Ok((msg_id, recv_ctx, data, _consumed)) => {
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
