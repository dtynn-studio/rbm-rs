use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};

use crate::{
    ensure_buf_size, ensure_ok,
    network::{ConnectionType, NetworkType},
    proto::{
        v1::{cset::CMD_SET_CTRL, impl_v1_cmd, V1},
        Deserialize, Serialize,
    },
    Result,
};

impl_v1_cmd!(SetSdkConnection, SetSdkConnectionResp, CMD_SET_CTRL, 0xd4);

#[derive(Debug)]
pub struct SetSdkConnection {
    // this field is not used anywhere in the origin sdk
    ctrl: u8,
    pub host: u8,
    pub network: NetworkType,
    pub connection: ConnectionType,
    pub addr: SocketAddrV4,
}

impl Default for SetSdkConnection {
    fn default() -> Self {
        Self {
            ctrl: 0,
            host: 0,
            network: NetworkType::default(),
            connection: ConnectionType::default(),
            addr: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 10010),
        }
    }
}

impl Serialize<V1> for SetSdkConnection {
    const SIZE_HINT: usize = 10;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let ip = self.addr.ip().octets();
        let port_bytes = self.addr.port().to_le_bytes();
        let data: [u8; Self::SIZE_HINT] = [
            self.ctrl,
            self.host,
            self.network as u8,
            self.connection as u8,
            ip[0],
            ip[1],
            ip[2],
            ip[3],
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

impl Deserialize<V1> for SetSdkConnectionResp {
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
