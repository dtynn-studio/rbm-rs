macro_rules! impl_module {
    ($name:ident) => {
        pub struct $name<CODEC: $crate::proto::Codec, C: $crate::client::Client<CODEC>> {
            client: std::sync::Arc<C>,

            _codec: std::marker::PhantomData<CODEC>,
        }

        impl<CODEC: $crate::proto::Codec, C: $crate::client::Client<CODEC>> $name<CODEC, C> {
            pub fn new(client: std::sync::Arc<C>) -> $crate::Result<Self> {
                Ok(Self {
                    client,
                    _codec: Default::default(),
                })
            }
        }
    };
}

pub(super) use impl_module;
