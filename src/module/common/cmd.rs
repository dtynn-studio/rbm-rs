use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};

use byteorder::WriteBytesExt;

use crate::{
    ensure_buf_size, ensure_ok,
    network::{ConnectionType, NetworkType},
    proto::{
        v1::{
            cset::{CMD_SET_COMMON, CMD_SET_CTRL},
            impl_v1_cmd, impl_v1_empty_ser, RetOK, V1,
        },
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

impl_v1_cmd!(EnableSdkMode, RetOK, CMD_SET_CTRL, 0xd1);

#[derive(Debug)]
pub struct EnableSdkMode(bool);

impl From<bool> for EnableSdkMode {
    #[inline]
    fn from(v: bool) -> Self {
        Self(v)
    }
}

impl Serialize<V1> for EnableSdkMode {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.0 as u8).map_err(From::from)
    }
}

impl_v1_cmd!(GetVersion, GetVersionResp, CMD_SET_COMMON, 1);

#[derive(Default, Debug)]
pub struct GetVersion;

impl_v1_empty_ser!(GetVersion);

#[derive(Debug)]
pub struct GetVersionResp {
    pub aa: u8,
    pub bb: u8,
    pub cc: u8,
    pub dd: u8,
    pub build: u8,
    pub version: u8,
    pub minor: u8,
    pub major: u8,
    pub cmds: u8,
    pub rooback: u8,
}

impl Default for GetVersionResp {
    fn default() -> Self {
        GetVersionResp {
            aa: 0,
            bb: 1,
            cc: 0,
            dd: 0,
            build: 1,
            version: 0,
            minor: 1,
            major: 0,
            cmds: 0,
            rooback: 0,
        }
    }
}

impl Deserialize<V1> for GetVersionResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);

        ensure_buf_size!(buf, 30);

        Ok(GetVersionResp {
            // TODO: why use buf[0] as _aa here?
            // see https://github.com/dji-sdk/RoboMaster-SDK/blob/8f301fd1bd3038f51c403614c52abbf9e9f5103c/src/robomaster/protocol.py#L435-L438
            aa: buf[0],
            bb: buf[1],
            cc: buf[2],
            dd: buf[3],
            ..Default::default()
        })
    }
}

impl_v1_cmd!(
    GetProductVersion,
    GetProductVersionResp,
    CMD_SET_COMMON,
    0x4f
);

#[derive(Debug)]
pub struct GetProductVersion {
    pub file_type: u8,
}

impl Default for GetProductVersion {
    fn default() -> Self {
        Self { file_type: 4 }
    }
}

impl Serialize<V1> for GetProductVersion {
    const SIZE_HINT: usize = 9;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let data: [u8; Self::SIZE_HINT] = [self.file_type, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff];
        w.write_all(&data[..]).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct GetProductVersionResp {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}

impl Deserialize<V1> for GetProductVersionResp {
    fn de(buf: &[u8]) -> Result<Self> {
        // see https://github.com/dji-sdk/RoboMaster-SDK/blob/8f301fd1bd3038f51c403614c52abbf9e9f5103c/src/robomaster/protocol.py#L463
        ensure_ok!(buf);

        ensure_buf_size!(buf, 9 + 4);
        Ok(GetProductVersionResp {
            major: buf[12],
            minor: buf[11],
            patch: u16::from_le_bytes([buf[9], buf[10]]),
        })
    }
}

impl_v1_cmd!(GetSN, GetSNResp, CMD_SET_COMMON, 0x51);

#[derive(Debug)]
pub struct GetSN {
    pub typ: u8,
}

impl Default for GetSN {
    fn default() -> Self {
        Self { typ: 1 }
    }
}

impl Serialize<V1> for GetSN {
    const SIZE_HINT: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.typ]).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct GetSNResp {
    pub sn: String,
}

impl Deserialize<V1> for GetSNResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);

        ensure_buf_size!(&buf[1..], 2, "sn response header");
        let sn_len = buf[1] as usize;
        let sn = String::from_utf8_lossy(&buf[3..3 + sn_len]).to_string();
        Ok(Self { sn })
    }
}
