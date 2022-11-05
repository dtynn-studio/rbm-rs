use std::io::{Cursor, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::{
    ensure_buf_size, ensure_ok,
    proto::{impl_empty_ser, v1::impl_v1_cmd, Deserialize, RetOK, Serialize},
    Error, Result,
};

use super::impl_v1_event;

const CMD_SET: u8 = 0x0a;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VisionType {
    Shoulder = 0,
    Person = 1,
    Gesture = 2,
    Line = 4,
    Marker = 5,
    Robot = 7,
}

impl TryFrom<u8> for VisionType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => VisionType::Shoulder,
            1 => VisionType::Person,
            2 => VisionType::Gesture,
            4 => VisionType::Line,
            5 => VisionType::Marker,
            7 => VisionType::Robot,
            other => {
                return Err(Error::InvalidData(
                    format!("invalid vision type {}", other).into(),
                ))
            }
        })
    }
}

impl_v1_cmd!(VisionDetectStatus, VisionTypeMask, 0xa5);

#[derive(Debug, Clone, Copy, Default)]
pub struct VisionTypeMask(pub u16);

impl VisionTypeMask {
    pub fn set(self, typ: VisionType) -> Self {
        VisionTypeMask(self.0 | (1 << typ as u8))
    }

    pub fn is_set(&self, typ: VisionType) -> bool {
        (self.0 & (1 << typ as u8)) != 0
    }
}

impl Deserialize for VisionTypeMask {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_ok!(buf);
        ensure_buf_size!(buf, 3);
        Ok(VisionTypeMask(buf[1] as u16 | (buf[2] as u16) << 8))
    }
}

#[derive(Debug, Default)]
pub struct VisionDetectStatus;

impl_empty_ser!(VisionDetectStatus);

impl_v1_cmd!(VisionSetColor, RetOK, 0xab);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VisionColorType {
    Line = 1,
    Marker = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VisionColor {
    Red = 1,
    Green = 2,
    Blue = 3,
}

#[derive(Debug)]
pub struct VisionSetColor {
    pub typ: VisionColorType,
    pub color: VisionColor,
}

impl Serialize for VisionSetColor {
    const SIZE: usize = 2;

    fn ser(&self, w: &mut impl std::io::Write) -> Result<()> {
        w.write_all(&[self.typ as u8, self.color as u8])
            .map_err(From::from)
    }
}

impl_v1_cmd!(VisionDetectEnable, RetOK, 0xa3);

#[derive(Debug)]
pub struct VisionDetectEnable(pub VisionType);

impl Serialize for VisionDetectEnable {
    const SIZE: usize = 2;

    fn ser(&self, w: &mut impl Write) -> Result<()> {
        w.write_u16::<LE>(self.0 as u16).map_err(From::from)
    }
}

impl_v1_event!(VisionDetectInfo, 0xa4);

#[derive(Debug)]
pub enum VisionRectInfo {
    Shoulder(Vec<[f32; 4]>),
    Person(Vec<[f32; 4]>),
    Gesture(Vec<([f32; 4], u32)>),
    Line(u32, Vec<([f32; 4])>),
    Marker(Vec<([f32; 4], u16)>),
    Robot(Vec<[f32; 4]>),
}

#[derive(Debug)]
pub struct VisionDetectInfo {
    pub typ: VisionType,
    pub status: u8,
    pub errcode: u16,
    pub rect_info: VisionRectInfo,
}

fn round(v: f32, precision: i32) -> f32 {
    let prec = 10f32.powi(precision);
    (v * prec).round() / prec
}

impl Deserialize for VisionDetectInfo {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 9);
        let typ: VisionType = buf[0].try_into()?;
        let status = buf[1];
        let errcode = buf[6] as u16 | (buf[7] as u16) << 8;
        let count = buf[8] as usize;
        ensure_buf_size!(buf, 9 + count * 20);
        let chunks = buf[9..].chunks_exact(20);
        let rect_info = match typ {
            VisionType::Shoulder => {
                let rects = chunks
                    .map(|data| {
                        let mut reader = Cursor::new(data);
                        let x = reader.read_f32::<LE>()?;
                        let y = reader.read_f32::<LE>()?;
                        let w = reader.read_f32::<LE>()?;
                        let h = reader.read_f32::<LE>()?;
                        Ok([round(x, 5), round(y, 5), round(w, 5), round(h, 5)])
                    })
                    .collect::<Result<Vec<_>>>()?;
                VisionRectInfo::Shoulder(rects)
            }

            VisionType::Person => {
                let rects = chunks
                    .map(|data| {
                        let mut reader = Cursor::new(data);
                        let x = reader.read_f32::<LE>()?;
                        let y = reader.read_f32::<LE>()?;
                        let w = reader.read_f32::<LE>()?;
                        let h = reader.read_f32::<LE>()?;
                        Ok([round(x, 5), round(y, 5), round(w, 5), round(h, 5)])
                    })
                    .collect::<Result<Vec<_>>>()?;
                VisionRectInfo::Person(rects)
            }

            VisionType::Gesture => {
                let rects = chunks
                    .map(|data| {
                        let mut reader = Cursor::new(data);
                        let x = reader.read_f32::<LE>()?;
                        let y = reader.read_f32::<LE>()?;
                        let w = reader.read_f32::<LE>()?;
                        let h = reader.read_f32::<LE>()?;
                        let info = reader.read_u32::<LE>()?;
                        Ok(([round(x, 5), round(y, 5), round(w, 5), round(h, 5)], info))
                    })
                    .collect::<Result<Vec<_>>>()?;
                VisionRectInfo::Gesture(rects)
            }

            VisionType::Line => {
                let mut ident = 0;
                let mut rects = vec![];
                for (i, data) in chunks.enumerate() {
                    let mut reader = Cursor::new(data);
                    let x = reader.read_f32::<LE>()?;
                    let y = reader.read_f32::<LE>()?;
                    let theta = reader.read_f32::<LE>()?;
                    let c = reader.read_f32::<LE>()?;
                    if i == 0 {
                        let info = reader.read_u32::<LE>()?;
                        ident = info;
                    }

                    rects.push([round(x, 7), round(y, 7), round(theta, 7), round(c, 7)]);
                }
                VisionRectInfo::Line(ident, rects)
            }

            VisionType::Marker => {
                let rects = chunks
                    .map(|data| {
                        let mut reader = Cursor::new(data);
                        let x = reader.read_f32::<LE>()?;
                        let y = reader.read_f32::<LE>()?;
                        let w = reader.read_f32::<LE>()?;
                        let h = reader.read_f32::<LE>()?;
                        let info = reader.read_u16::<LE>()?;
                        Ok(([round(x, 5), round(y, 5), round(w, 5), round(h, 5)], info))
                    })
                    .collect::<Result<Vec<_>>>()?;

                VisionRectInfo::Marker(rects)
            }

            VisionType::Robot => {
                let rects = chunks
                    .map(|data| {
                        let mut reader = Cursor::new(data);
                        let x = reader.read_f32::<LE>()?;
                        let y = reader.read_f32::<LE>()?;
                        let w = reader.read_f32::<LE>()?;
                        let h = reader.read_f32::<LE>()?;
                        Ok([round(x, 5), round(y, 5), round(w, 5), round(h, 5)])
                    })
                    .collect::<Result<Vec<_>>>()?;
                VisionRectInfo::Robot(rects)
            }
        };

        Ok(Self {
            typ,
            status,
            errcode,
            rect_info,
        })
    }
}
