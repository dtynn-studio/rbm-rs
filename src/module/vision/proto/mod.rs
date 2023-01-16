use std::convert::TryFrom;

use crate::{util::macros::impl_num_enums, Error};

pub mod cmd;
pub mod sub;

impl_num_enums!(
    DetectType,
    u16,
    Shoulder = 0,
    Person = 1,
    Gesture = 2,
    Line = 4,
    Marker = 5,
    Robot = 7,
);

impl DetectType {
    #[inline]
    fn mask(self) -> u16 {
        1 << self as u16
    }
}

impl_num_enums!(
    Gesture,
    Jump = 1,
    LeftHandUp = 2,
    RightHandUp = 3,
    Victory = 4,
    GiveIn = 5,
    Capture = 6,
    LeftHandWave = 7,
    RightHandWave = 8,
    Idle = 9,
);

impl_num_enums!(
    MarkerShape,
    Red = 1,
    Yellow = 2,
    Green = 3,
    Left = 4,
    Right = 5,
    Forward = 6,
    Backward = 7,
    Heart = 8,
    Sword = 9,
    ExclamationMark = 46,
    QuestionMark = 47,
    HashTag = 48,
);

#[derive(Debug, Clone, Copy)]
pub enum Marker {
    Number(u8),
    // upper case
    Letter(u8),
    Shape(MarkerShape),
}

impl TryFrom<u8> for Marker {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            10..=19 => Marker::Number(value - 10),
            20..=45 => Marker::Letter(value + 65 - 20),
            other => return MarkerShape::try_from(other).map(Marker::Shape),
        })
    }
}
