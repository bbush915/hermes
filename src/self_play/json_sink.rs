use std::io::Write;

use crate::core::EventSink;
use crate::self_play::sample::Sample;

use serde_json::to_writer;

pub struct JsonSink<W: Write> {
    writer: W,
}

impl<W: Write> JsonSink<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> EventSink<Sample> for JsonSink<W> {
    fn emit(&mut self, sample: Sample) {
        to_writer(&mut self.writer, &sample).unwrap();

        writeln!(&mut self.writer).unwrap();
    }
}
