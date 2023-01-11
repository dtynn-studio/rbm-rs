macro_rules! impl_v1_msg {
    ($name:ident, $cset:ident, $cid:literal) => {
        impl $crate::proto::ProtoMessage<$crate::proto::v1::V1> for $name {
            const IDENT: $crate::proto::v1::Ident = ($cset, $cid);
        }
    };
}

macro_rules! impl_v1_action_update {
    ($name:ident, $cset:ident, $cid:literal) => {
        impl $crate::proto::ProtoPush<$crate::proto::v1::V1> for $name {
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
        impl $crate::proto::Deserialize<$crate::proto::v1::V1> for $name {
            fn de(_buf: &[u8]) -> $crate::Result<Self> {
                Ok(Self::default())
            }
        }
    };
}

macro_rules! impl_v1_sub_self {
    ($name:ident) => {
        impl_v1_sub_self!(
            $name,
            $crate::module::common::constant::v1::Uid::$name as u64
        );
    };

    ($name:ty, $uid:expr) => {
        impl $crate::proto::ProtoSubscribe<$crate::proto::v1::V1> for $name {
            const SID: u64 = $uid;

            type Push = $name;

            fn apply_push(&mut self, push: Self::Push) -> Result<()> {
                let _ = std::mem::replace(self, push);
                Ok(())
            }
        }
    };
}

pub(crate) use impl_v1_action_update;
pub(crate) use impl_v1_cmd;
pub(crate) use impl_v1_empty_de;
pub(crate) use impl_v1_empty_ser;
pub(crate) use impl_v1_msg;
pub(crate) use impl_v1_sub_self;
