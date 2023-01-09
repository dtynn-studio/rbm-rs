use std::io::{Read, Write};

use crate::Result;

pub trait ReadOrderedExt<Out>: Read {
    fn read_le(&mut self) -> Result<Out>;

    fn read_be(&mut self) -> Result<Out>;
}

pub trait WriteOrderedExt<In>: Write {
    fn write_le(&mut self, val: In) -> Result<()>;
    fn write_be(&mut self, val: In) -> Result<()>;
}

impl<T: Read> ReadOrderedExt<u8> for T {
    #[inline]
    fn read_le(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_be(&mut self) -> Result<u8> {
        self.read_le()
    }
}

impl<T: Read> ReadOrderedExt<i8> for T {
    #[inline]
    fn read_le(&mut self) -> Result<i8> {
        self.read_le().map(|v: u8| v as i8)
    }

    #[inline]
    fn read_be(&mut self) -> Result<i8> {
        self.read_le()
    }
}

impl<T: Write> WriteOrderedExt<u8> for T {
    #[inline]
    fn write_le(&mut self, val: u8) -> Result<()> {
        self.write_all(&[val]).map_err(From::from)
    }

    #[inline]
    fn write_be(&mut self, val: u8) -> Result<()> {
        self.write_le(val)
    }
}

impl<T: Write> WriteOrderedExt<i8> for T {
    #[inline]
    fn write_le(&mut self, val: i8) -> Result<()> {
        self.write_le(val as u8)
    }

    #[inline]
    fn write_be(&mut self, val: i8) -> Result<()> {
        self.write_be(val as u8)
    }
}

macro_rules! impl_read_write_ordered {
    ($nty:ty, $size:literal) => {
        impl<T: Read> ReadOrderedExt<$nty> for T {
            #[inline]
            fn read_le(&mut self) -> Result<$nty> {
                let mut buf = [0u8; $size];
                self.read_exact(&mut buf)?;
                Ok(<$nty>::from_le_bytes(buf))
            }

            #[inline]
            fn read_be(&mut self) -> Result<$nty> {
                let mut buf = [0u8; $size];
                self.read_exact(&mut buf)?;
                Ok(<$nty>::from_be_bytes(buf))
            }
        }

        impl<T: Write> WriteOrderedExt<$nty> for T {
            #[inline]
            fn write_le(&mut self, val: $nty) -> Result<()> {
                self.write_all(val.to_le_bytes().as_ref())
                    .map_err(From::from)
            }

            #[inline]
            fn write_be(&mut self, val: $nty) -> Result<()> {
                self.write_all(val.to_be_bytes().as_ref())
                    .map_err(From::from)
            }
        }
    };
}

impl_read_write_ordered!(i16, 2);
impl_read_write_ordered!(u16, 2);
impl_read_write_ordered!(i32, 4);
impl_read_write_ordered!(u32, 4);
impl_read_write_ordered!(i64, 8);
impl_read_write_ordered!(u64, 8);

impl_read_write_ordered!(f32, 4);
impl_read_write_ordered!(f64, 8);
