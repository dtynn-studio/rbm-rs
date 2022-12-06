macro_rules! impl_v1_msg {
    ($name:ident, $cset:ident, $cid:literal) => {
        impl $crate::proto::ProtoMessage<$crate::proto::v1::V1> for $name {
            const IDENT: $crate::proto::v1::Ident = ($cset, $cid);
        }
    };
}

macro_rules! impl_v1_action_update {
    ($name:ident, $cset:ident, $cid:literal) => {
        impl $crate::proto::v1::action::V1ActionUpdate for $name {
            const IDENT: $crate::proto::v1::Ident = ($cset, $cid);
        }
    };
}

macro_rules! impl_v1_cmd {
    ($name:ident, $resp:ty, $cset:ident, $cid:literal) => {
        $crate::proto::v1::impl_v1_msg!($name, $cset, $cid);

        impl $crate::proto::ProtoCommand<$crate::proto::v1::V1> for $name {
            type Resp = $resp;
        }
    };
}

macro_rules! impl_v1_empty_ser {
    ($name:ty) => {
        impl $crate::proto::Serialize<$crate::proto::v1::V1> for $name {
            const SIZE_HINT: usize = 0;

            fn ser(&self, _w: &mut impl std::io::Write) -> $crate::Result<()> {
                Ok(())
            }
        }
    };
}

macro_rules! impl_v1_empty_de {
    ($name:ty) => {
        impl $crate::proto::Deserialize for $name {
            fn de(_buf: &[u8]) -> $crate::Result<Self> {
                Ok(Self::default())
            }
        }
    };
}

pub(crate) use impl_v1_action_update;
pub(crate) use impl_v1_cmd;
pub(crate) use impl_v1_empty_de;
pub(crate) use impl_v1_empty_ser;
pub(crate) use impl_v1_msg;
