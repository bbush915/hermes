pub trait EventSink<E> {
    fn emit(&mut self, event: E);
}
