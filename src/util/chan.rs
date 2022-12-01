use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

use crate::{Error, Result};

pub fn unbounded<T>() -> (Tx<T>, Rx<T>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let arc = Arc::new(rx);
    (
        Tx {
            tx,
            rx: arc.clone(),
        },
        Rx { rx: arc },
    )
}

pub fn bounded<T>(cap: usize) -> (Tx<T>, Rx<T>) {
    let (tx, rx) = crossbeam_channel::bounded(cap);
    let arc = Arc::new(rx);
    (
        Tx {
            tx,
            rx: arc.clone(),
        },
        Rx { rx: arc },
    )
}

pub struct Tx<T> {
    tx: Sender<T>,
    rx: Arc<Receiver<T>>,
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
        Arc::strong_count(&self.rx) == 1
    }
}

pub struct Rx<T> {
    rx: Arc<Receiver<T>>,
}

impl<T> Rx<T> {
    pub fn recv(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }

    pub fn inner(&mut self) -> &Receiver<T> {
        self.rx.as_ref()
    }

    pub fn is_closed(&mut self) -> bool {
        Arc::strong_count(&self.rx) == 1
    }
}
