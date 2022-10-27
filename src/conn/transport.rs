use std::io::{self, Read, Write};
use std::net::{TcpStream, UdpSocket};

pub trait Transport: Send + Sync + Sized {
    fn send(&mut self, data: &[u8]) -> io::Result<()>;
    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn try_clone(&self) -> io::Result<Self>;
}

pub struct TcpTransport {
    inner: TcpStream,
}

impl Transport for TcpTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.inner.write_all(data)
    }

    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn try_clone(&self) -> io::Result<Self> {
        self.inner.try_clone().map(|inner| TcpTransport { inner })
    }
}

pub struct UdpTransport {
    inner: UdpSocket,
}

impl Transport for UdpTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.inner.send(data)?;
        Ok(())
    }

    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.recv(buf)
    }

    fn try_clone(&self) -> io::Result<Self> {
        self.inner.try_clone().map(|inner| UdpTransport { inner })
    }
}
