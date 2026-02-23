use std::io::Write;

use serde_json::to_writer;

use crate::core::EventSink;
use crate::self_play::sample::Sample;

pub struct JsonSampleSink<W: Write> {
    writer: W,
}

impl<W: Write> JsonSampleSink<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> EventSink<Sample> for JsonSampleSink<W> {
    fn emit(&mut self, sample: Sample) {
        to_writer(&mut self.writer, &sample).expect("unable to write sample");

        writeln!(&mut self.writer).expect("unable to write newline");
    }
}
