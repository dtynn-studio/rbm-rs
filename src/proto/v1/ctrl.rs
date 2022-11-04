use std::{
    io::{Cursor, Write},
    net::Ipv4Addr,
};

use byteorder::{WriteBytesExt, LE};

use crate::{
    conn::{ConnectionType, NetworkType},
    ensure_buf_size, ensure_ok,
    proto::{
        impl_empty_ser,
        v1::{impl_v1_cmd, impl_v1_event},
        Deserialize, DussMBType, RetOK, Serialize,
    },
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
        w.write_all(&[self.0.into()]).map_err(From::from)
    }
}

impl_v1_cmd!(ChassisStickOverlay, RetOK, 0x28);

#[derive(Debug, Default)]
pub struct ChassisStickOverlay {
    pub mode: u8,
}

impl Serialize for ChassisStickOverlay {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.mode]).map_err(From::from)
    }
}

impl_v1_event!(ArmorHitEvent, 0x2);

#[derive(Debug)]
pub struct ArmorHitEvent {
    pub index: u8,
    pub typ: u8,
    pub mic_value: u16,
    pub mic_len: u16,
}

impl Deserialize for ArmorHitEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 5);

        Ok(Self {
            index: buf[0] >> 4,
            typ: buf[0] & 0xf,
            mic_value: u16::from_le_bytes(buf[1..3].try_into().unwrap()),
            mic_len: u16::from_le_bytes(buf[3..5].try_into().unwrap()),
        })
    }
}

impl_v1_event!(IrHitEvent, 0x10);

#[derive(Debug)]
pub struct IrHitEvent {
    pub role_id: u8,
    pub skill_id: u8,
    pub recv_dev: u8,
    pub recv_ir_pin: u8,
}

impl Deserialize for IrHitEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 3);
        Ok(Self {
            role_id: buf[0] >> 4,
            skill_id: buf[0] & 0xf,
            recv_dev: buf[1],
            recv_ir_pin: buf[2],
        })
    }
}

impl_v1_event!(GameMsgEvent, 0xd6);

#[derive(Debug)]
pub struct GameMsgEvent {
    pub buf: Vec<u8>,
}

impl Deserialize for GameMsgEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 1);
        let size = buf[0] as usize;

        ensure_buf_size!(&buf[1..], size);
        Ok(Self {
            buf: buf[1..].into(),
        })
    }
}

impl_v1_cmd!(SetArmorParam, RetOK, 0x7);

#[derive(Debug, Default)]
pub struct SetArmorParam {
    armor_mask: u8,
    voice_energy_en: u16,
    voice_energy_ex: u16,
    voice_len_max: u16,
    voice_len_min: u16,
    voice_len_silence: u16,
    voice_peak_count: u16,
    voice_peak_min: u16,
    voice_peak_ave: u16,
    voice_peak_final: u16,
}

impl Serialize for SetArmorParam {
    const SIZE: usize = 19;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE];
        let mut cursor = Cursor::new(&mut buf[..]);
        cursor.write_u8(self.armor_mask)?;
        cursor.write_u16::<LE>(self.voice_energy_en)?;
        cursor.write_u16::<LE>(self.voice_energy_ex)?;
        cursor.write_u16::<LE>(self.voice_len_max)?;
        cursor.write_u16::<LE>(self.voice_len_min)?;
        cursor.write_u16::<LE>(self.voice_len_silence)?;
        cursor.write_u16::<LE>(self.voice_peak_count)?;
        cursor.write_u16::<LE>(self.voice_peak_min)?;
        cursor.write_u16::<LE>(self.voice_peak_ave)?;
        cursor.write_u16::<LE>(self.voice_peak_final)?;
        w.write_all(&buf[..]).map_err(From::from)
    }
}

impl_v1_cmd!(ChassisWheelSpeed, RetOK, 0x26);

#[derive(Debug, Default)]
pub struct ChassisWheelSpeed {
    pub w1_spd: u8,
    pub w2_spd: u8,
    pub w3_spd: u8,
    pub w4_spd: u8,
}

impl Serialize for ChassisWheelSpeed {
    const SIZE: usize = 4;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.w1_spd, self.w2_spd, self.w3_spd, self.w4_spd])
            .map_err(From::from)
    }
}

impl_v1_cmd!(SetSystemLed, RetOK, 0x33);

#[derive(Debug)]
pub struct SetSystemLed {
    pub comp_mask: u32,
    pub led_mask: i16,
    pub ctrl_mode: u8,
    pub effect_mode: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub loop_: u8,
    pub t1: i16,
    pub t2: i16,
}

impl Default for SetSystemLed {
    fn default() -> Self {
        Self {
            comp_mask: 0x3f,
            led_mask: 0xff,
            ctrl_mode: 0,
            effect_mode: 0,
            r: 0xff,
            g: 0xff,
            b: 0xff,
            loop_: 0,
            t1: 100,
            t2: 100,
        }
    }
}

impl Serialize for SetSystemLed {
    const SIZE: usize = 15;
    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE];
        Cursor::new(&mut buf[0..4]).write_u32::<LE>(self.comp_mask)?;
        Cursor::new(&mut buf[4..6]).write_i16::<LE>(self.led_mask)?;

        buf[6] = self.ctrl_mode << 4 | self.effect_mode;
        buf[7] = self.r;
        buf[8] = self.g;
        buf[9] = self.b;
        buf[10] = self.loop_;

        Cursor::new(&mut buf[11..13]).write_i16::<LE>(self.t1)?;
        Cursor::new(&mut buf[13..15]).write_i16::<LE>(self.t2)?;

        w.write_all(&buf[..]).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct RobotMode(pub u8);

impl From<u8> for RobotMode {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl Deserialize for RobotMode {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 2);
        Ok(buf[1].into())
    }
}

impl_v1_cmd!(SetRobotMode, RetOK, 0x46);

#[derive(Debug)]
pub struct SetRobotMode(pub RobotMode);

impl Default for SetRobotMode {
    fn default() -> Self {
        Self(1.into())
    }
}

impl Serialize for SetRobotMode {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.0 .0).map_err(From::from)
    }
}

impl_v1_cmd!(GetRobotMode, RobotMode, 0x47);

#[derive(Debug)]
pub struct GetRobotMode;

impl_empty_ser!(GetRobotMode);

impl_v1_cmd!(BlasterFire, RetOK, 0x51);

#[derive(Debug, Default)]
pub struct BlasterFire {
    pub typ: u8,
    pub times: u8,
}

impl Serialize for BlasterFire {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.typ << 4 | self.times).map_err(From::from)
    }
}

impl_v1_cmd!(BlasterSetLed, RetOK, 0x55, DussMBType::Push);

#[derive(Debug)]
pub struct BlasterSetLed {
    pub mode: u8,
    pub effect: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub times: u8,
    pub t1: u16,
    pub t2: u16,
}

impl Default for BlasterSetLed {
    fn default() -> Self {
        Self {
            mode: 7,
            effect: 0,
            r: 0xff,
            g: 0xff,
            b: 0xff,
            times: 1,
            t1: 100,
            t2: 100,
        }
    }
}

impl Serialize for BlasterSetLed {
    const SIZE: usize = 9;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let mut buf = [0u8; Self::SIZE];
        buf[0] = self.mode << 4 | self.effect;
        buf[1] = self.r;
        buf[2] = self.g;
        buf[3] = self.b;
        buf[4] = self.times;
        Cursor::new(&mut buf[4..6]).write_u16::<LE>(self.t1)?;
        Cursor::new(&mut buf[6..8]).write_u16::<LE>(self.t2)?;
        w.write_all(&buf[..]).map_err(From::from)
    }
}
