use std::marker::PhantomData;
use std::sync::Arc;

use tracing::warn;

use crate::{client::Connection, proto::Codec, Result};

pub struct Vision<CODEC: Codec, C: Connection<CODEC>> {
    client: Arc<C>,

    _codec: PhantomData<CODEC>,
}

impl<CODEC: Codec, C: Connection<CODEC>> Vision<CODEC, C> {
    pub fn reset(&mut self) -> Result<()> {
        warn!("vision reset is not implemented yet");
        Ok(())
    }
}
