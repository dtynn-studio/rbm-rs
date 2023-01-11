use std::io::Cursor;

use super::{DetectType, Gesture, Marker};
use crate::{
    ensure_buf_size,
    proto::{
        v1::{cset::CMD_SET_VISION, Ident, V1},
        Deserialize, ProtoPush,
    },
    util::{decimal::round, ordered::ReadOrderedExt},
    Result,
};

const RECT_ROUND_DIGITS: i32 = 5;
const RECT_INFO_SIZE: usize = 16;

#[derive(Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline]
    fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let x = reader.read_le()?;
        let y = reader.read_le()?;
        let w = reader.read_le()?;
        let h = reader.read_le()?;
        Ok(Rect {
            x: round(x, RECT_ROUND_DIGITS),
            y: round(y, RECT_ROUND_DIGITS),
            width: round(w, RECT_ROUND_DIGITS),
            height: round(h, RECT_ROUND_DIGITS),
        })
    }
}

#[derive(Debug)]
pub struct MarkerRect {
    pub typ: Marker,
    pub rect: Rect,
    pub distance: u16,
}

#[derive(Debug)]
pub struct GestureRect {
    pub typ: Gesture,
    pub rect: Rect,
}

#[derive(Debug)]
pub struct LineRect {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
    pub c: f32,
}

// :rect_info: 包含的信息如下：
//         person 行人识别：(x, y, w, h), x 中心点x轴坐标，y 中心点y轴坐标，w 宽度，h 高度
//         gesture 手势识别：(x, y, w, h), x 中心点x轴坐标，y 中心点y轴坐标，w 宽度，h 高度
//         line 线识别：(x, y, theta, C)，x点x轴坐标，y点y轴坐标，theta切线角，C 曲率
//         marker 识别：(x, y, w, h, marker), x 中心点x轴坐标，y 中心点y轴坐标，w 宽度，h 高度，marker 识别到的标签
//         robot 机器人识别：(x, y, w, h)，x 中心点x轴坐标，y 中心点y轴坐标，w 宽度，h 高度

#[derive(Debug)]
pub enum RectInfo {
    Shoulder(Vec<Rect>),
    Person(Vec<Rect>),
    Gesture(Vec<GestureRect>),
    Line(u32, Vec<LineRect>),
    Marker(Vec<MarkerRect>),
    Robot(Vec<Rect>),
}

#[derive(Debug)]
pub struct DetectInfo {
    pub typ: DetectType,
    pub status: u8,
    pub errcode: u16,
    pub rect_info: RectInfo,
}

impl ProtoPush<V1> for DetectInfo {
    const IDENT: Ident = (CMD_SET_VISION, 0xa4);
}

impl Deserialize<V1> for DetectInfo {
    fn de(buf: &[u8]) -> Result<Self> {
        ensure_buf_size!(buf, 9);
        let typ = DetectType::try_from(buf[0] as u16)?;
        let status = buf[1];
        let errcode = buf[6] as u16 | (buf[7] as u16) << 8;
        let count = buf[8] as usize;
        ensure_buf_size!(buf, 9 + count * 20);
        let chunks = buf[9..].chunks_exact(20);
        let rect_info = match typ {
            DetectType::Shoulder => {
                let rects = chunks.map(Rect::from_bytes).collect::<Result<Vec<_>>>()?;
                RectInfo::Shoulder(rects)
            }

            DetectType::Person => {
                let rects = chunks.map(Rect::from_bytes).collect::<Result<Vec<_>>>()?;
                RectInfo::Person(rects)
            }

            DetectType::Gesture => {
                let rects = chunks
                    .map(|data| {
                        let rect = Rect::from_bytes(data)?;
                        let mut reader = Cursor::new(&data[RECT_INFO_SIZE..]);
                        let info: u32 = reader.read_le()?;
                        Ok(GestureRect {
                            rect,
                            typ: Gesture::try_from(info as u8)?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                RectInfo::Gesture(rects)
            }

            DetectType::Line => {
                let mut ident = 0;
                let mut rects = vec![];
                for (i, data) in chunks.enumerate() {
                    let mut reader = Cursor::new(data);
                    let x = reader.read_le()?;
                    let y = reader.read_le()?;
                    let theta = reader.read_le()?;
                    let c = reader.read_le()?;
                    if i == 0 {
                        let info: u32 = reader.read_le()?;
                        ident = info;
                    }

                    rects.push(LineRect {
                        x: round(x, 7),
                        y: round(y, 7),
                        theta: round(theta, 7),
                        c: round(c, 7),
                    });
                }
                RectInfo::Line(ident, rects)
            }

            DetectType::Marker => {
                let rects = chunks
                    .map(|data| {
                        let rect = Rect::from_bytes(data)?;
                        let mut reader = Cursor::new(&data[RECT_INFO_SIZE..]);
                        let info: u16 = reader.read_le()?;
                        let distance = reader.read_le()?;
                        Ok(MarkerRect {
                            typ: Marker::try_from(info as u8)?,
                            rect,
                            distance,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                RectInfo::Marker(rects)
            }

            DetectType::Robot => {
                let rects = chunks.map(Rect::from_bytes).collect::<Result<Vec<_>>>()?;
                RectInfo::Robot(rects)
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
