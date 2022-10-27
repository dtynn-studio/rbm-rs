use std::io::Write;

use crate::{
    ensure_buf_size, ensure_ok,
    proto::{impl_empty_ser, v1::impl_v1_cmd, Deserialize, Serialize},
    Result,
};

const CMD_SET: u8 = 0x00;

impl_v1_cmd!(GetVersion, GetVersionResp, 1);

#[derive(Default, Debug)]
pub struct GetVersion;

impl_empty_ser!(GetVersion);

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

impl Deserialize for GetVersionResp {
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

impl_v1_cmd!(GetProductVersion, GetProductVersionResp, 0x4f);

#[derive(Debug)]
pub struct GetProductVersion {
    pub file_type: u8,
}

impl Default for GetProductVersion {
    fn default() -> Self {
        Self { file_type: 4 }
    }
}

impl Serialize for GetProductVersion {
    const SIZE: usize = 9;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let data: [u8; Self::SIZE] = [self.file_type, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff];
        w.write_all(&data[..]).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct GetProductVersionResp {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}

impl Deserialize for GetProductVersionResp {
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

impl_v1_cmd!(GetSN, GetSNResp, 0x51);

#[derive(Debug)]
pub struct GetSN {
    pub typ: u8,
}

impl Default for GetSN {
    fn default() -> Self {
        Self { typ: 1 }
    }
}

impl Serialize for GetSN {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.typ]).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct GetSNResp {
    pub sn: String,
}

impl Deserialize for GetSNResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);

        ensure_buf_size!(&buf[1..], 2, "sn response header");
        let sn_len = buf[1] as usize;
        let sn = String::from_utf8_lossy(&buf[3..3 + sn_len]).to_string();
        Ok(Self { sn })
    }
}
