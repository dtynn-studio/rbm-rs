use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpStream, UdpSocket};
use std::sync::Arc;

use net2::TcpBuilder;

pub mod udp;

pub trait TransportTx: Send + Sync {
    fn send(&mut self, data: &[u8]) -> Result<()>;
}

pub trait TransportRx: Send + Sync {
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait TransportRxCloser: Send + Sync {
    fn close(&mut self) -> Result<()>;
}

pub trait Transport: Send + Sync + Sized {
    const CONTINUOUS: bool;

    fn connect(bind: Option<SocketAddr>, dest: SocketAddr) -> Result<Self>;

    fn send(&mut self, data: &[u8]) -> Result<()>;
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn try_clone(&self) -> Result<Self>;
    fn shutdown(&mut self);
}

pub struct Tcp {
    inner: TcpStream,
}

impl Transport for Tcp {
    const CONTINUOUS: bool = true;

    fn connect(bind: Option<SocketAddr>, dest: SocketAddr) -> Result<Self> {
        let builder = TcpBuilder::new_v4()?;

        if let Some(bind) = bind {
            builder.bind(bind)?;
        }

        builder
            .connect(dest)
            .map(|inner| Tcp { inner })
            .map_err(From::from)
    }

    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.inner.write_all(data)
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }

    fn try_clone(&self) -> Result<Self> {
        self.inner.try_clone().map(|inner| Tcp { inner })
    }

    fn shutdown(&mut self) {
        let _ = self.inner.shutdown(Shutdown::Both);
    }
}

pub struct Udp {
    inner: Option<Arc<UdpSocket>>,
    local: SocketAddr,
    dest: SocketAddr,
}

impl Transport for Udp {
    const CONTINUOUS: bool = false;

    fn connect(bind: Option<SocketAddr>, dest: SocketAddr) -> Result<Self> {
        let socket = if let Some(bind) = bind {
            UdpSocket::bind(bind)?
        } else {
            UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0))?
        };

        let local = socket.local_addr()?;

        Ok(Udp {
            inner: Some(Arc::new(socket)),
            local,
            dest,
        })
    }

    fn send(&mut self, data: &[u8]) -> Result<()> {
        match self.inner.as_ref() {
            Some(inner) => inner.send_to(data, self.dest).map(|_| ()),
            None => Err(Error::new(ErrorKind::NotConnected, "socket dropped")),
        }
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.inner.as_ref() {
            Some(inner) => inner
                .recv_from(buf)
                .map(|(read, from)| if from == self.local { 0 } else { read })
                .or_else(|e| {
                    if e.kind() == ErrorKind::WouldBlock {
                        return Ok(0);
                    }

                    Err(e)
                }),
            None => Err(Error::new(ErrorKind::NotConnected, "socket dropped")),
        }
    }

    fn try_clone(&self) -> Result<Self> {
        match self.inner.as_ref() {
            Some(inner) => Ok(Udp {
                inner: Some(inner.clone()),
                local: self.local,
                dest: self.dest,
            }),
            None => Err(Error::new(ErrorKind::NotConnected, "socket dropped")),
        }
    }

    fn shutdown(&mut self) {
        if let Some(inner) = self.inner.take() {
            // some tricks to trigger exit on recv side;
            let _ = inner.send_to(&[0xff], self.local);
            let _ = inner.set_nonblocking(true);
        }
    }
}
