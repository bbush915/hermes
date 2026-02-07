use crate::core::{EventSink, Game, RunnerEvent};

pub struct NullSink;

impl NullSink {
    pub fn new() -> Self {
        NullSink
    }
}

impl<G: Game> EventSink<RunnerEvent<G>> for NullSink {
    fn emit(&mut self, _event: RunnerEvent<G>) {}
}
