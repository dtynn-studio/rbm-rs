use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam_channel::{Receiver, Sender};

use crate::{Error, Result};

#[inline]
fn wrap_chan<T>(tx: Sender<T>, rx: Receiver<T>) -> (Tx<T>, Rx<T>) {
    let arc_tx = Arc::new(tx);
    let closed = Arc::new(AtomicBool::default());
    (
        Tx {
            tx: arc_tx.clone(),
            closed: closed.clone(),
        },
        Rx {
            rx,
            tx: arc_tx,
            closed,
        },
    )
}

pub fn unbounded<T>() -> (Tx<T>, Rx<T>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    wrap_chan(tx, rx)
}

pub fn bounded<T>(cap: usize) -> (Tx<T>, Rx<T>) {
    let (tx, rx) = crossbeam_channel::bounded(cap);
    wrap_chan(tx, rx)
}

#[derive(Clone)]
pub struct Tx<T> {
    tx: Arc<Sender<T>>,
    closed: Arc<AtomicBool>,
}

impl<T> Tx<T> {
    pub fn send(&mut self, ele: T) -> Result<()> {
        self.tx
            .send(ele)
            .map_err(|_e| Error::Other("chan broken".into()))
    }

    pub fn inner(&mut self) -> &Sender<T> {
        &self.tx
    }

    pub fn is_closed(&mut self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }
}

impl<T> Drop for Rx<T> {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}

pub struct Rx<T> {
    rx: Receiver<T>,
    tx: Arc<Sender<T>>,
    closed: Arc<AtomicBool>,
}

impl<T> Rx<T> {
    pub fn recv(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }

    pub fn inner(&mut self) -> &Receiver<T> {
        &self.rx
    }

    pub fn is_closed(&mut self) -> bool {
        Arc::strong_count(&self.tx) == 1
    }
}
