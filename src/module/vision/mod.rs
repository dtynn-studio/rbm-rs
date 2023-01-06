use tracing::warn;

use crate::{client::Client, proto::Codec, Result};

use super::impl_module;

impl_module!(Vision);

impl<CODEC: Codec, C: Client<CODEC>> Vision<CODEC, C> {
    pub fn reset(&mut self) -> Result<()> {
        warn!("vision reset is not implemented yet");
        Ok(())
    }
}
