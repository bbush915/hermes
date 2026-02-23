use crate::core::event::EventSink;
use crate::core::game::Game;
use crate::core::runner::runner::{RunnerEvent, RunnerEventContext, RunnerEventKind};

#[derive(Default)]
pub struct StdoutRunnerEventSink;

impl StdoutRunnerEventSink {
    pub fn new() -> Self {
        StdoutRunnerEventSink
    }
}

impl<G: Game> EventSink<RunnerEvent<G>> for StdoutRunnerEventSink {
    fn emit(&mut self, event: RunnerEvent<G>) {
        let RunnerEvent { kind, context } = event;

        let Some(RunnerEventContext {
            game,
            game_number,
            turn,
            turn_number,
        }) = context
        else {
            return;
        };

        match kind {
            RunnerEventKind::GameStarted => {
                println!("=== Game #{} ===\n", game_number + 1);
            }
            RunnerEventKind::TurnStarted => {
                println!("--- Turn #{} ---\n", turn_number + 1);
            }
            RunnerEventKind::ActionApplied { action } => {
                println!("{turn:?} {action}\n");
                println!("{}", game.display(turn));
            }
            RunnerEventKind::GameFinished { outcome } => {
                println!("{}", outcome.display(turn));
            }
            _ => {}
        }
    }
}
