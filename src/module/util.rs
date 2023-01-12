macro_rules! impl_module {
    ($name:ident $(,$fname:ident : $ftype:ty)* $(,~ $dfname:ident : $dftype:ty)*) => {
        pub struct $name<CODEC: $crate::proto::Codec, C: $crate::client::Client<CODEC>> {
            client: std::sync::Arc<C>,

            _codec: std::marker::PhantomData<CODEC>,

            $(
                $fname: $ftype,
            )*

            $(
                $dfname: $dftype,
            )*
        }

        impl<CODEC: $crate::proto::Codec, C: $crate::client::Client<CODEC>> $name<CODEC, C> {
            pub fn new(
                client: std::sync::Arc<C>,
                $(
                    $fname: $ftype,
                )*
            ) -> $crate::Result<Self> {
                Ok(Self {
                    client,
                    _codec: Default::default(),
                    $(
                        $fname,
                     )*
                    $(
                        $dfname: Default::default(),
                     )*
                })
            }
        }
    };
}

macro_rules! impl_v1_subscribe_meth_simple {
    ($subty:ty) => {
        paste::paste! {
            impl_v1_subscribe_meth_simple!([<$subty:snake>], $subty);
        }
    };

    ($meth:ident, $subty:ty) => {
        paste::paste! {
            pub fn [<subscribe_ $meth>](
                &mut self,
                freq: Option<$crate::proto::v1::subscribe::SubFreq>,
            ) -> $crate::Result<(
                $subty,
                $crate::util::chan::Rx<$subty>,
                Box<dyn $crate::client::Subscription<$crate::proto::v1::V1>>,
            )> {
                self.client
                    .subscribe_period_push::<$subty>(freq)
                    .map(|(rx, sub)| (Default::default(), rx, sub))
            }
        }
    };
}

pub(super) use impl_module;
pub(super) use impl_v1_subscribe_meth_simple;
