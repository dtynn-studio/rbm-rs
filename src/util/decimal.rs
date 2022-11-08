use std::fmt::Display;

use crate::{Error, Result};

pub trait MaybeRound: Copy + Display {
    fn round(self, digits: i32) -> Result<Self> {
        if digits == 0 {
            return Ok(self);
        }

        Err(Error::InvalidData(
            format!("invalid value to be rounded {}", self).into(),
        ))
    }
}

macro_rules! impl_maybe_round_default {
    ($first:ty $(, $tails:ty )+) => {
        impl_maybe_round_default!($first);

        $(
            impl_maybe_round_default!($tails);
         )+

    };

    ($ty:ty) => {
        impl MaybeRound for $ty {}
    };
}

impl_maybe_round_default!(i16, u16);

impl MaybeRound for f32 {
    fn round(self, digits: i32) -> Result<Self> {
        if digits == 0 {
            return Ok(self);
        }

        Ok(round(self, digits))
    }
}

pub fn round(v: f32, digits: i32) -> f32 {
    let prec = 10f32.powi(digits);
    (v * prec).round() / prec
}

pub trait MaybeFrom<T>: Sized {
    fn maybe_from(v: T) -> Result<Self>;
}

pub trait MaybeInto<T> {
    fn maybe_into(self) -> Result<T>;
}

impl<T, T1> MaybeInto<T1> for T
where
    T1: MaybeFrom<T>,
{
    #[inline]
    fn maybe_into(self) -> Result<T1> {
        T1::maybe_from(self)
    }
}

macro_rules! impl_maybe_move {
    ($first:ty $(, $tails:ty )+) => {
        impl_maybe_move!($first);
        $(
            impl_maybe_move!($tails);
         )+
    };

    ($t:ty) => {
        impl MaybeFrom<$t> for $t {
            #[inline]
            fn maybe_from(v: $t) -> Result<Self> {
                Ok(v)
            }
        }
    };
}

impl_maybe_move!(f32, i16, u16);

macro_rules! impl_maybe_as {
    ($t1:ty, $t2:ty) => {
        impl MaybeFrom<$t2> for $t1 {
            #[inline]
            fn maybe_from(v: $t2) -> Result<Self> {
                Ok(v as $t1)
            }
        }

        impl MaybeFrom<$t1> for $t2 {
            #[inline]
            fn maybe_from(v: $t1) -> Result<Self> {
                Ok(v as $t2)
            }
        }
    };
}

impl_maybe_as!(f32, i16);
impl_maybe_as!(f32, u8);
