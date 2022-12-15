use super::{Codec, Deserialize, ProtoCommand};

/// ProtoSubscribe: ask for the events published by the target, the incoming stream of events will not
/// be terminated until user unsub the events.
pub trait ProtoSubscribe<C: Codec> {
    type Cmd: ProtoCommand<C>;
    type Push: Deserialize<C>;
}
