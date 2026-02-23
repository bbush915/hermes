use crate::core::EventSink;

#[derive(Default)]
pub struct NullEventSink;

impl NullEventSink {
    pub fn new() -> Self {
        NullEventSink
    }
}

impl<E> EventSink<E> for NullEventSink {
    fn emit(&mut self, _event: E) {}
}
