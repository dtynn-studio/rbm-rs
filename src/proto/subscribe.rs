use super::{Codec, Deserialize};
use crate::Result;

/// ProtoSubscribe: ask for the events published by the target, the incoming stream of events will not
/// be terminated until user unsub the events.
pub trait ProtoSubscribe<C: Codec> {
    type Push: Deserialize<C>;

    fn apply_push(&mut self, push: Self::Push) -> Result<()>;
}
