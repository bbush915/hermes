#[allow(clippy::module_inception)]
mod runner;
mod statistics_runner_event_sink;
mod stdout_runner_event_sink;

pub use runner::{Runner, RunnerEvent, RunnerEventContext, RunnerEventKind};
pub use statistics_runner_event_sink::StatisticsRunnerEventSink;
pub use stdout_runner_event_sink::StdoutRunnerEventSink;
