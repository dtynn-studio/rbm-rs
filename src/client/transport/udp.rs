use std::io::{Error, ErrorKind, Result};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::Arc;

use super::{TransportRx, TransportRxCloser, TransportTx};

pub fn trans_tx_to(socket: Arc<UdpSocket>, dest: SocketAddr) -> Box<dyn TransportTx> {
    Box::new((socket, dest))
}

pub fn trans_rx(
    socket: Arc<UdpSocket>,
) -> Result<(Box<dyn TransportRx>, Box<dyn TransportRxCloser>)> {
    let mut local = socket.local_addr()?;
    let local_ip = local.ip();
    if local_ip.is_unspecified() {
        if local_ip.is_ipv4() {
            local.set_ip(Ipv4Addr::LOCALHOST.into());
        } else {
            local.set_ip(Ipv6Addr::LOCALHOST.into());
        }
    }

    Ok((
        Box::new((Some(socket.clone()), local)),
        Box::new((Some(socket), local)),
    ))
}

impl TransportTx for (Arc<UdpSocket>, SocketAddr) {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.0.send_to(data, self.1).map(|_| ())
    }
}

impl TransportRx for (Option<Arc<UdpSocket>>, SocketAddr) {
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = match self.0.as_ref() {
            Some(inner) => inner
                .recv_from(buf)
                .map(
                    |(read, from)| {
                        if from == self.1 {
                            None
                        } else {
                            Some(read)
                        }
                    },
                )
                .or_else(|e| {
                    if e.kind() == ErrorKind::WouldBlock {
                        Ok(Some(0))
                    } else {
                        Err(e)
                    }
                })?,

            None => {
                return Err(Error::new(
                    ErrorKind::NotConnected,
                    "transport receiver dropped",
                ))
            }
        };

        if let Some(read) = read {
            return Ok(read);
        }

        self.0.take();
        Err(Error::new(
            ErrorKind::NotConnected,
            "transport receiver closed",
        ))
    }
}

impl TransportRxCloser for (Option<Arc<UdpSocket>>, SocketAddr) {
    fn close(&mut self) -> Result<()> {
        if let Some(inner) = self.0.take() {
            inner.send_to(&[0xff], self.1)?;
        }

        Ok(())
    }
}
