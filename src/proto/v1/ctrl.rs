use std::{
    io::{Cursor, Write},
    net::Ipv4Addr,
};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::{
    algo::{crc16_calc, crc8_calc},
    conn::{ConnectionType, NetworkType},
    ensure_buf_size, ensure_ok,
    proto::{
        host2byte, impl_empty_ser,
        v1::{impl_v1_action_cmd, impl_v1_cmd, impl_v1_event},
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

// see https://github.com/dji-sdk/RoboMaster-SDK/blob/8f301fd1bd3038f51c403614c52abbf9e9f5103c/src/robomaster/chassis.py#L353-L355
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ChassisStickOverlayMode {
    Disabled = 0,
    ChassisMode = 1,
    GimbalMode = 2,
}

#[derive(Debug)]
pub struct ChassisStickOverlay {
    pub mode: ChassisStickOverlayMode,
}

impl Serialize for ChassisStickOverlay {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.mode as u8]).map_err(From::from)
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

impl_v1_cmd!(StreamCtrl, RetOK, 0xd2);

#[derive(Debug)]
pub struct StreamCtrl {
    pub ctrl: u8,
    pub conn_type: u8,
    pub state: u8,
    pub resolution: u8,
}

impl Default for StreamCtrl {
    fn default() -> Self {
        Self {
            ctrl: 1,
            conn_type: 0,
            state: 1,
            resolution: 0,
        }
    }
}

impl Serialize for StreamCtrl {
    const SIZE: usize = 3;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[self.ctrl, self.conn_type << 4 | self.state, self.resolution])
            .map_err(From::from)
    }
}

impl_v1_cmd!(SdkHeartBeat, RetOK, 0xd5);

#[derive(Debug, Default)]
pub struct SdkHeartBeat;

impl_empty_ser!(SdkHeartBeat);

impl_v1_event!(AiModuleEvent, 0xea);

#[derive(Debug)]
pub struct AiModuleEventItem {
    pub id: u8,
    pub x: u16,
    pub y: u8,
    pub w: u16,
    pub h: u8,
    pub c: u8,
}

#[derive(Debug)]
pub struct AiModuleEvent {
    pub info: Vec<AiModuleEventItem>,
}

impl Deserialize for AiModuleEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 15);
        let item_num = (buf.len() - 15) / 8;
        ensure_buf_size!(buf, 15 + 8 * item_num);

        let info = (buf[13..13 + 8 * item_num])
            .chunks(8)
            .map(|data| {
                let mut reader = Cursor::new(data);
                let id = reader.read_u8()?;
                let x = reader.read_u16::<LE>()?;
                let y = reader.read_u8()?;
                let w = reader.read_u16::<LE>()?;
                let h = reader.read_u8()?;
                let c = reader.read_u8()?;
                Ok(AiModuleEventItem { id, x, y, w, h, c })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { info })
    }
}

impl_v1_event!(UwbModuleEvent, 0xdb);

#[derive(Debug, Default)]
pub struct UwbModuleEvent {
    pub id: u8,
    pub pox_x: f32,
    pub pox_y: f32,
    pub pox_z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub eop_x: u8,
    pub eop_y: u8,
    pub eop_z: u8,
}

impl Deserialize for UwbModuleEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 28);
        let mut reader = Cursor::new(buf);
        let id = reader.read_u8()?;
        let pox_x = reader.read_f32::<LE>()?;
        let pox_y = reader.read_f32::<LE>()?;
        let pox_z = reader.read_f32::<LE>()?;
        let vel_x = reader.read_f32::<LE>()?;
        let vel_y = reader.read_f32::<LE>()?;
        let vel_z = reader.read_f32::<LE>()?;
        let eop_x = reader.read_u8()?;
        let eop_y = reader.read_u8()?;
        let eop_z = reader.read_u8()?;

        Ok(Self {
            id,
            pox_x,
            pox_y,
            pox_z,
            vel_x,
            vel_y,
            vel_z,
            eop_x,
            eop_y,
            eop_z,
        })
    }
}

impl_v1_action_cmd!(PlaySound, 0xb3);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PlaySoundCtrl {
    Stop = 0,
    Interupt = 1,
    Mixed = 2,
    Ignored = 3,
}

#[derive(Debug)]
pub struct PlaySound {
    pub action_id: u8,
    pub push_freq: u8,
    pub task_ctrl: u8,
    pub sound_id: u32,
    pub play_ctrl: PlaySoundCtrl,
    pub interval: u16,
    pub play_times: u8,
}

impl Default for PlaySound {
    fn default() -> Self {
        Self {
            action_id: 0,
            push_freq: 2,
            task_ctrl: 0,
            sound_id: 0,
            play_ctrl: PlaySoundCtrl::Interupt,
            interval: 0,
            play_times: 0,
        }
    }
}

impl Serialize for PlaySound {
    const SIZE: usize = 10;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.action_id)?;
        w.write_u8(self.task_ctrl | (self.push_freq << 2))?;
        w.write_u32::<LE>(self.sound_id)?;
        w.write_u8(self.play_ctrl as u8)?;
        w.write_u16::<LE>(self.interval)?;
        w.write_u8(self.play_times)?;
        Ok(())
    }
}

impl_v1_event!(SoundPushEvent, 0xb4);

#[derive(Debug, Default)]
pub struct SoundPushEvent {
    pub reserved: u8,
    pub sound_id: u32,
}

impl Deserialize for SoundPushEvent {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 4);

        let sound_id = Cursor::new(buf).read_u32::<LE>()?;

        Ok(Self {
            reserved: 0,
            sound_id,
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ActionCtrl {
    Start = 0,
    Cancel = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ActionPushFreq {
    OneHZ = 0,
    FiveHZ = 1,
    TenHz = 2,
}

impl_v1_action_cmd!(GimbalRotate, 0xb0);

#[derive(Debug)]
pub struct GimbalRotate {
    pub action_id: u8,
    pub action_ctrl: ActionCtrl,
    pub push_freq: ActionPushFreq,
    pub coordinate: u8,
    pub yaw_valid: bool,
    pub roll_valid: bool,
    pub pitch_valid: bool,
    pub error: u16,
    pub yaw: i16,   // Unit: 0.1 degree
    pub roll: i16,  // Unit: 0.1 degree
    pub pitch: i16, // Unit: 0.1 degree
    pub yaw_speed: u16,
    pub roll_speed: u16,
    pub pitch_speed: u16,
}

impl Default for GimbalRotate {
    fn default() -> Self {
        Self {
            action_id: 0,
            action_ctrl: ActionCtrl::Start,
            push_freq: ActionPushFreq::TenHz,
            coordinate: 3,
            yaw_valid: true,
            roll_valid: false,
            pitch_valid: true,
            error: 0,
            yaw: 0,
            roll: 0,
            pitch: 0,
            yaw_speed: 30,
            roll_speed: 0,
            pitch_speed: 30,
        }
    }
}

impl Serialize for GimbalRotate {
    const SIZE: usize = 17;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.action_id)?;
        w.write_u8(self.action_ctrl as u8 | (self.push_freq as u8) << 2)?;
        w.write_u8(
            self.yaw_valid as u8
                | (self.roll_valid as u8) << 1
                | (self.pitch_valid as u8) << 2
                | (self.coordinate << 3),
        )?;

        w.write_i16::<LE>(self.yaw)?;
        w.write_i16::<LE>(self.roll)?;
        w.write_i16::<LE>(self.pitch)?;

        w.write_u16::<LE>(self.error)?;

        w.write_u16::<LE>(self.yaw_speed)?;
        w.write_u16::<LE>(self.roll_speed)?;
        w.write_u16::<LE>(self.pitch_speed)?;

        Ok(())
    }
}

impl_v1_event!(GimbalActionPush, 0xb1);

#[derive(Debug, Default)]
pub struct GimbalActionPush {
    pub yaw: i16,
    pub roll: i16,
    pub pitch: i16,
}

impl Deserialize for GimbalActionPush {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 6);

        let mut reader = Cursor::new(buf);
        let yaw = reader.read_i16::<LE>()?;
        let roll = reader.read_i16::<LE>()?;
        let pitch = reader.read_i16::<LE>()?;

        Ok(Self { yaw, roll, pitch })
    }
}

impl_v1_action_cmd!(GimbalRecenter, 0xb2);

#[derive(Debug)]
pub struct GimbalRecenter {
    pub action_id: u8,
    pub action_ctrl: ActionCtrl,
    pub push_freq: ActionPushFreq,
    pub yaw_valid: bool,
    pub roll_valid: bool,
    pub pitch_valid: bool,
    pub yaw_speed: u16,
    pub roll_speed: u16,
    pub pitch_speed: u16,
}

impl Default for GimbalRecenter {
    fn default() -> Self {
        Self {
            action_id: 0,
            action_ctrl: ActionCtrl::Start,
            push_freq: ActionPushFreq::TenHz,
            yaw_valid: true,
            roll_valid: false,
            pitch_valid: true,
            yaw_speed: 100,
            roll_speed: 0,
            pitch_speed: 100,
        }
    }
}

impl Serialize for GimbalRecenter {
    const SIZE: usize = 9;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.action_id)?;
        w.write_u8(self.action_ctrl as u8 | (self.push_freq as u8) << 2)?;
        w.write_u8(
            self.yaw_valid as u8 | (self.roll_valid as u8) << 1 | (self.pitch_valid as u8) << 2,
        )?;

        w.write_u16::<LE>(self.yaw_speed)?;
        w.write_u16::<LE>(self.roll_speed)?;
        w.write_u16::<LE>(self.pitch_speed)?;

        Ok(())
    }
}

impl_v1_action_cmd!(PositionMove, 0x25);

#[derive(Debug)]
pub struct PositionMove {
    pub action_id: u8,
    pub freq: ActionPushFreq,
    pub action_ctrl: ActionCtrl,
    pub ctrl_mode: u8,
    pub axis_mode: u8,
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
    pub vel_xy_max: u8,
    pub agl_omg_max: i16,
}

impl Default for PositionMove {
    fn default() -> Self {
        Self {
            action_id: 0,
            freq: ActionPushFreq::TenHz,
            action_ctrl: ActionCtrl::Start,
            ctrl_mode: 0,
            axis_mode: 0,
            pos_x: 0,
            pos_y: 0,
            pos_z: 0,
            vel_xy_max: 0,
            agl_omg_max: 300,
        }
    }
}

impl Serialize for PositionMove {
    const SIZE: usize = 13;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.action_id)?;
        w.write_u8(self.action_ctrl as u8 | (self.freq as u8) << 2)?;
        w.write_u8(self.ctrl_mode)?;
        w.write_u8(self.axis_mode)?;

        w.write_i16::<LE>(self.pos_x)?;
        w.write_i16::<LE>(self.pos_y)?;
        w.write_i16::<LE>(self.pos_z)?;

        w.write_u8(self.vel_xy_max)?;

        w.write_i16::<LE>(self.agl_omg_max)?;

        Ok(())
    }
}

impl_v1_event!(PositionPush, 0x2a);

#[derive(Debug)]
pub struct PositionPush {
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
}

impl Deserialize for PositionPush {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 6);
        let mut reader = Cursor::new(buf);
        let pos_x = reader.read_i16::<LE>()?;
        let pos_y = reader.read_i16::<LE>()?;
        let pos_z = reader.read_i16::<LE>()?;

        Ok(Self {
            pos_x,
            pos_y,
            pos_z,
        })
    }
}

impl_v1_cmd!(SetWheelSpeed, RetOK, 0x20);

#[derive(Debug, Default)]
pub struct SetWheelSpeed {
    pub w1_spd: i16,
    pub w2_spd: i16,
    pub w3_spd: i16,
    pub w4_spd: i16,
}

impl Serialize for SetWheelSpeed {
    const SIZE: usize = 8;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_i16::<LE>(self.w1_spd)?;
        w.write_i16::<LE>(self.w2_spd)?;
        w.write_i16::<LE>(self.w3_spd)?;
        w.write_i16::<LE>(self.w4_spd)?;
        Ok(())
    }
}

impl_v1_cmd!(ChassisSetWorkMode, RetOK, 0x19);

#[derive(Debug, Default)]
pub struct ChassisSetWorkMode {
    pub mode: u8,
}

impl Serialize for ChassisSetWorkMode {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.mode).map_err(From::from)
    }
}

impl_v1_cmd!(ChassisSpeedMode, RetOK, 0x21, DussMBType::Push);

#[derive(Debug, Default)]
pub struct ChassisSpeedMode {
    pub x_spd: f32,
    pub y_spd: f32,
    pub z_spd: f32,
}

impl Serialize for ChassisSpeedMode {
    const SIZE: usize = 12;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_f32::<LE>(self.x_spd)?;
        w.write_f32::<LE>(self.y_spd)?;
        w.write_f32::<LE>(self.z_spd)?;
        Ok(())
    }
}

impl_v1_cmd!(ChassisPwmPercent, RetOK, 0x3c);

#[derive(Debug, Default)]
pub struct ChassisPwmPercent {
    pub mask: u8,
    pub pwms: [u16; 6],
}

impl Serialize for ChassisPwmPercent {
    const SIZE: usize = 13;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.mask)?;
        for v in self.pwms {
            w.write_u16::<LE>(v)?;
        }
        Ok(())
    }
}

impl_v1_cmd!(ChassisPwmFreq, RetOK, 0x2b);

#[derive(Debug, Default)]
pub struct ChassisPwmFreq {
    pub mask: u8,
    pub pwms: [u16; 6],
}

impl Serialize for ChassisPwmFreq {
    const SIZE: usize = 13;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.mask)?;
        for v in self.pwms {
            w.write_u16::<LE>(v)?;
        }
        Ok(())
    }
}

impl_v1_cmd!(ChassisSerialSet, RetOK, 0xc0);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SerialBaudRate {
    Rate9600 = 0,
    Rate19200 = 1,
    Rate38400 = 2,
    Rate57600 = 3,
    Rate115200 = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SerialDataBit {
    Bit7 = 0,
    Bit8 = 1,
    Bit9 = 2,
    Bit10 = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SerialOddEven {
    None = 0,
    Odd = 1,
    Even = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SerialStopBit {
    One = 1,
    Two = 2,
}

#[derive(Debug)]
pub struct ChassisSerialSet {
    pub baud_rate: SerialBaudRate,
    pub data_bit: SerialDataBit,
    pub odd_even: SerialOddEven,
    pub stop_bit: SerialStopBit,
    pub tx_enabled: bool,
    pub rx_enabled: bool,
    pub rx_size: u16,
    pub tx_size: u16,
}

// see: https://github.com/dji-sdk/RoboMaster-SDK/blob/8f301fd1bd3038f51c403614c52abbf9e9f5103c/src/robomaster/uart.py#L105-L131
impl Default for ChassisSerialSet {
    fn default() -> Self {
        Self {
            baud_rate: SerialBaudRate::Rate9600,
            data_bit: SerialDataBit::Bit8,
            odd_even: SerialOddEven::None,
            stop_bit: SerialStopBit::One,
            tx_enabled: true,
            rx_enabled: true,
            tx_size: 50,
            rx_size: 50,
        }
    }
}

impl Serialize for ChassisSerialSet {
    const SIZE: usize = 6;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(
            (self.stop_bit as u8 & 0x1) << 7
                | (self.odd_even as u8 & 0x3) << 5
                | (self.data_bit as u8 & 0x3) << 3
                | (self.baud_rate as u8 & 0x7),
        )?;

        // should be tx_enabled & rx_enabled?
        w.write_u8(0xff)?;

        w.write_u16::<LE>(self.rx_size)?;
        w.write_u16::<LE>(self.tx_size)?;
        Ok(())
    }
}

impl_v1_cmd!(ChassisSerialMsgSend, RetOK, 0xc1);

#[derive(Debug)]
pub struct ChassisSerialMsgSend {
    pub typ: u8,
    pub msg: Vec<u8>,
}

impl ChassisSerialMsgSend {
    pub fn new(msg: Vec<u8>) -> Self {
        Self { typ: 0x2, msg }
    }
}

impl Serialize for ChassisSerialMsgSend {
    const SIZE: usize = 3;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.typ)?;
        w.write_u16::<LE>(self.msg.len() as u16)?;
        w.write_all(&self.msg)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        Self::SIZE + self.msg.len()
    }
}

impl_v1_cmd!(SensorGetData, SensorGetDataResp, 0xf0);

#[derive(Debug)]
pub struct SensorGetDataResp {
    pub port: u8,
    pub adc: u16,
    pub io: u8,
    pub time: u32,
}

impl Deserialize for SensorGetDataResp {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 9);
        let mut reader = Cursor::new(&buf[1..]);
        let port = reader.read_u8()?;
        let adc = reader.read_u16::<LE>()?;
        let io = reader.read_u8()?;
        let time = reader.read_u32::<LE>()?;

        Ok(Self {
            port,
            adc,
            io,
            time,
        })
    }
}

#[derive(Debug)]
pub struct SensorGetData {
    pub port: u8,
}

impl Serialize for SensorGetData {
    const SIZE: usize = 1;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.port).map_err(From::from)
    }
}

impl_v1_action_cmd!(ServoCtrlSet, 0xb7);

#[derive(Debug)]
pub struct ServoCtrlSet {
    pub action_id: u8,
    pub freq: ActionPushFreq,
    pub action_ctrl: u8,
    pub id: u8,
    pub value: i32,
}

impl Default for ServoCtrlSet {
    fn default() -> Self {
        Self {
            action_id: 0,
            freq: ActionPushFreq::TenHz,
            action_ctrl: 0,
            id: 0,
            value: 0,
        }
    }
}

impl Serialize for ServoCtrlSet {
    const SIZE: usize = 7;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.action_id)?;
        w.write_u8(self.action_ctrl | (self.freq as u8) << 2)?;
        w.write_u8(host2byte(25, self.id))?;
        w.write_i32::<LE>(self.value)?;

        Ok(())
    }
}

impl_v1_event!(ServoCtrlPush, 0xb8);

#[derive(Debug)]
pub struct ServoCtrlPush {
    pub value: i32,
}

impl Deserialize for ServoCtrlPush {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 7);
        let value = Cursor::new(buf).read_i32::<LE>()?;
        Ok(Self { value })
    }
}

impl_v1_action_cmd!(RoboticArmMoveCtrl, 0xb5);

#[derive(Debug)]
pub struct RoboticArmMoveCtrl {
    pub action_id: u8,
    pub freq: ActionPushFreq,
    pub action_ctrl: ActionCtrl,
    pub id: u8,
    pub mode: u8,
    pub mask: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for RoboticArmMoveCtrl {
    fn default() -> Self {
        Self {
            action_id: 0,
            freq: ActionPushFreq::TenHz,
            action_ctrl: ActionCtrl::Start,
            id: host2byte(27, 2),
            mode: 0,
            mask: 0x3,
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Serialize for RoboticArmMoveCtrl {
    const SIZE: usize = 17;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u8(self.action_id)?;
        w.write_u8(self.action_ctrl as u8 | ((self.freq as u8) << 2))?;
        w.write_u8(self.id)?;
        w.write_u8(self.mode)?;
        w.write_u8(self.mask)?;
        w.write_i32::<LE>(self.x)?;
        w.write_i32::<LE>(self.y)?;
        w.write_i32::<LE>(self.z)?;
        Ok(())
    }
}

impl_v1_event!(RoboticArmMovePush, 0xb6);

#[derive(Debug)]
pub struct RoboticArmMovePush {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Deserialize for RoboticArmMovePush {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 12);
        let mut reader = Cursor::new(buf);
        let x = reader.read_i32::<LE>()?;
        let y = reader.read_i32::<LE>()?;
        let z = reader.read_i32::<LE>()?;
        Ok(Self { x, y, z })
    }
}

impl_v1_cmd!(RoboticAiInit, (), 0xe9);

#[derive(Debug)]
pub struct RoboticAiInit {
    pub addr: u16,
    pub sender: u16,
    pub reciver: u16,
    pub seq_num: u16,
    pub cmd: u16,
    pub attr: u8,
}

impl Default for RoboticAiInit {
    fn default() -> Self {
        Self {
            addr: 0x0103,
            sender: 0x0103,
            reciver: 0x0301,
            seq_num: 0,
            cmd: 0x020d,
            attr: 0,
        }
    }
}

impl Serialize for RoboticAiInit {
    const SIZE: usize = 17;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        let len: u16 = 2;
        let mut buf = [0u8; Self::SIZE];

        buf[0] = 0xAA;
        buf[1] = (len & 0xff) as u8;
        buf[2] = (len >> 8) as u8;
        let crc_h = crc8_calc(&buf[0..3], Some(0x11));
        buf[3] = crc_h;
        buf[4] = (self.sender & 0xff) as u8;
        buf[5] = (self.sender >> 8) as u8;
        buf[6] = (self.reciver & 0xff) as u8;
        buf[7] = (self.reciver >> 8) as u8;
        buf[8] = self.attr;
        buf[9] = (self.seq_num & 0xff) as u8;
        buf[10] = (self.seq_num >> 8) as u8;
        buf[11] = (self.cmd & 0xff) as u8;
        buf[12] = (self.cmd >> 8) as u8;
        buf[13] = (self.addr & 0xff) as u8;
        buf[14] = (self.addr >> 8) as u8;
        let crc_h16 = crc16_calc(&buf[0..Self::SIZE - 2], Some(0x4F19));
        buf[15] = (crc_h16 & 0xff) as u8;
        buf[16] = (crc_h16 >> 8) as u8;

        w.write_all(&buf[..]).map_err(From::from)
    }
}
