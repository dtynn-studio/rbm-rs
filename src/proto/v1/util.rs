macro_rules! impl_v1_msg {
    ($name:ident, $cset:ident, $cid:literal) => {
        impl $crate::proto::ProtoMessage<crate::proto::v1::V1> for $name {
            const IDENT: $crate::proto::v1::Ident = ($cset, $cid);
        }
    };
}

macro_rules! impl_v1_cmd {
    ($name:ident, $resp:ty, $cset:ident, $cid:literal) => {
        $crate::proto::v1::impl_v1_msg!($name, $cset, $cid);

        impl $crate::proto::ProtoCommand<crate::proto::v1::V1> for $name {
            type Resp = $resp;
        }
    };
}

pub(crate) use impl_v1_cmd;
pub(crate) use impl_v1_msg;
