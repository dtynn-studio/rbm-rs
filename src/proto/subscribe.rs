use super::{Codec, Deserialize};
use crate::Result;

/// ProtoSubscribe: ask for the events published by the target, the incoming stream of events will not
/// be terminated until user unsub the events.
pub trait ProtoSubscribe<C: Codec> {
    const SID: C::SubscribeID;
    type Push: Deserialize<C> + Send + 'static;

    fn apply_push(&mut self, push: Self::Push) -> Result<()>;
}
