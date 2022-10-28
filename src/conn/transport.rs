use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpStream, UdpSocket};

use net2::TcpBuilder;

pub trait Transport: Send + Sync + Sized {
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
    fn connect(bind: Option<SocketAddr>, dest: SocketAddr) -> Result<Self> {
        let builder = if dest.is_ipv4() {
            TcpBuilder::new_v4()
        } else {
            TcpBuilder::new_v6()
        }?;

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
    inner: Option<UdpSocket>,
}

impl Transport for Udp {
    fn connect(bind: Option<SocketAddr>, dest: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(
            bind.unwrap_or_else(|| SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)),
        )?;

        socket.connect(dest)?;

        Ok(Udp {
            inner: Some(socket),
        })
    }

    fn send(&mut self, data: &[u8]) -> Result<()> {
        match self.inner.as_ref() {
            Some(inner) => inner.send(data).map(|_| ()),
            None => Err(Error::new(ErrorKind::NotConnected, "socket dropped")),
        }
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.inner.as_ref() {
            Some(inner) => inner.recv(buf),
            None => Err(Error::new(ErrorKind::NotConnected, "socket dropped")),
        }
    }

    fn try_clone(&self) -> Result<Self> {
        match self.inner.as_ref() {
            Some(inner) => inner.try_clone().map(|socket| Udp {
                inner: Some(socket),
            }),
            None => Err(Error::new(ErrorKind::NotConnected, "socket dropped")),
        }
    }

    fn shutdown(&mut self) {
        self.inner.take();
    }
}
