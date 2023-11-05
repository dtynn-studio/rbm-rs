//! see [struct](https://docs.python.org/3/library/struct.html#module-struct)
//! byte orders:
//!

use std::io::{Read, Result, Write};

use byteorder::ByteOrder;

mod numbers;

pub trait Pack: Sized {
    fn size(&self) -> Option<usize>;

    fn pack<T: ByteOrder>(&self, w: &mut impl Write) -> Result<()>;
}

pub trait Unpack: Sized {
    fn unpack<T: ByteOrder>(&mut self, r: &mut impl Read) -> Result<()>;
}

pub mod preclude {
    pub use super::{Pack, Unpack};
    pub use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt, BE, LE};
    pub use std::io::{Read, Result, Write};
}
