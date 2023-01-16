macro_rules! impl_num_enums {
    ($tname:ident, $($vname:ident = $val:expr,)+) => {
        impl_num_enums!($tname, u8, $($vname = $val,)+);
    };

    ($tname:ident, $nty:ty, $($vname:ident = $val:expr,)+) => {
        #[repr($nty)]
        #[derive(Debug, Clone, Copy)]
        pub enum $tname {
            $(
                $vname = $val,
             )+
        }

        impl std::convert::TryFrom<$nty> for $tname {
            type Error = $crate::Error;

            fn try_from(val: $nty) -> Result<Self, Self::Error> {
                Ok(match val {
                    $(
                        $val => $tname::$vname,
                     )+
                    other => return Err($crate::Error::Other(format!("unexpected value {} for {}", other, stringify!($tname)).into())),
                })
            }
        }
    };
}

pub(crate) use impl_num_enums;
