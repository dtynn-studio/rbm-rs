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

impl<E: Pack> Pack for &[E] {
    fn size(&self) -> Option<usize> {
        let mut size = 0;
        for v in &self[..] {
            size += v.size()?;
        }

        Some(size)
    }

    fn pack<T: ByteOrder>(&self, w: &mut impl Write) -> Result<()> {
        for v in &self[..] {
            v.pack::<T>(w)?;
        }
        Ok(())
    }
}

impl<E: Unpack> Unpack for &mut [E] {
    fn unpack<T: ByteOrder>(&mut self, r: &mut impl Read) -> Result<()> {
        for v in &mut self[..] {
            v.unpack::<T>(r)?;
        }

        Ok(())
    }
}

pub mod preclude {
    pub use super::{Pack, Unpack};
    pub use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt, BE, LE};
    pub use std::io::{Read, Result, Write};
}
