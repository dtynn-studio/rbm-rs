use crate::preclude::*;

use paste::paste;

macro_rules! impl_pack_unpack_num {
    ($($t:ty,)+ false) => {
        $(
        impl Pack for $t {
            #[inline]
            fn size(&self) -> Option<usize> {
                Some(std::mem::size_of::<$t>())
            }

            fn pack<T: ByteOrder>(&self, r: &mut impl Write) -> Result<()> {
                paste!(r.[<write_ $t>](*self))
            }

        }

        impl Unpack for $t {
            fn unpack<T: ByteOrder>(&mut self, r: &mut impl Read) -> Result<()> {
                let v = paste!(r.[<read_ $t>]())?;
                *self = v;
                Ok(())
            }

        }
         )+
    };

    ($($t:ty,)+ true) => {
        $(
        impl Pack for $t {
            #[inline]
            fn size(&self) -> Option<usize> {
                Some(std::mem::size_of::<$t>())
            }

            fn pack<T: ByteOrder>(&self, w: &mut impl Write) -> Result<()> {
                paste!(w.[<write_ $t>]::<T>(*self))
            }
        }

        impl Unpack for $t {
            fn unpack<T: ByteOrder>(&mut self, r: &mut impl Read) -> Result<()> {
                let v = paste!(r.[<read_ $t>]::<T>())?;
                *self = v;
                Ok(())
            }

        }
         )+
    };
}

impl_pack_unpack_num!(u8, i8, false);
impl_pack_unpack_num!(u16, i16, u32, i32, u64, i64, f32, f64, true);
