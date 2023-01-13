pub mod action;
pub mod cmd;
pub mod sub;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ServoIndex {
    No1 = 1,
    No2 = 2,
    No3 = 3,
}

impl ServoIndex {
    #[inline]
    fn into_idx(self) -> u8 {
        ((self as u8) << 5) + 0x19
    }
}
