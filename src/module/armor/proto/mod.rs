use crate::util::macros::impl_num_enums;

pub mod cmd;
pub mod sub;

impl_num_enums!(
    ArmorCompMask,
    TopLeft = 0b00100000,
    TopRight = 0b00010000,
    BottomLeft = 0b00001000,
    BottomRight = 0b00000100,
    BottomFront = 0b00000010,
    BottomBack = 0b00000001,
    BottomAll = 0x0f,
    TopAll = 0x30,
    All = 0x3f,
);

impl_num_enums!(
    ArmorId,
    BottomBack = 1,
    BottomFront = 2,
    BottomLeft = 3,
    BottomRight = 4,
    TopLeft = 5,
    TopRight = 6,
);
