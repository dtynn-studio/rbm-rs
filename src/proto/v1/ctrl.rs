use std::{io::Write, net::Ipv4Addr};

use crate::{
    conn::{ConnectionType, NetworkType},
    ensure_buf_size, ensure_ok,
    proto::{v1::impl_v1_cmd, Deserialize, RetOK, Serialize},
    Result,
};

const CMD_SET: u8 = 0x3f;

impl_v1_cmd!(SetSdkConnection, SetSdkConnectionResp, 0xd4);

#[derive(Debug)]
pub struct SetSdkConnection {
    pub ctrl: u8,
    pub host: u8,
    pub net_type: NetworkType,
    pub conn_type: ConnectionType,
    pub ip: [u8; 4],
    pub port: u16,
}

impl Default for SetSdkConnection {
    fn default() -> Self {
        Self {
            ctrl: 0,
            host: 0,
            net_type: NetworkType::default(),
            conn_type: ConnectionType::default(),
            ip: Ipv4Addr::UNSPECIFIED.octets(),
            port: 10010,
        }
    }
}

impl Serialize for SetSdkConnection {
    const SIZE: usize = 10;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let port_bytes = self.port.to_le_bytes();
        let data: [u8; Self::SIZE] = [
            self.ctrl,
            self.host,
            self.net_type as u8,
            self.conn_type as u8,
            self.ip[0],
            self.ip[1],
            self.ip[2],
            self.ip[3],
            port_bytes[0],
            port_bytes[1],
        ];
        w.write_all(&data[..]).map_err(From::from)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SetSdkConnectionResp {
    Accepted,
    Rejected,
    IP(Ipv4Addr),
    Other(u8),
}

impl Deserialize for SetSdkConnectionResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);

        ensure_buf_size!(&buf[1..], 1, "state");
        let state = buf[1];
        let resp = match state {
            0 => Self::Accepted,
            1 => Self::Rejected,
            2 => {
                ensure_buf_size!(&buf[2..], 4, "conn ip");
                Self::IP(Ipv4Addr::new(buf[2], buf[3], buf[4], buf[5]))
            }
            other => Self::Other(other),
        };

        Ok(resp)
    }
}

impl_v1_cmd!(SetSdkMode, RetOK, 0xd1);

#[derive(Debug)]
pub struct SetSdkMode(bool);

impl From<bool> for SetSdkMode {
    #[inline]
    fn from(v: bool) -> Self {
        Self(v)
    }
}

impl Serialize for SetSdkMode {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[if self.0 { 1 } else { 0 }])
            .map_err(From::from)
    }
}
