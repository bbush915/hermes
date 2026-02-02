use crate::core::{EventSink, Game, RunnerEvent};

pub struct StdoutSink;

impl StdoutSink {
    pub fn new() -> Self {
        StdoutSink
    }
}

impl<G: Game> EventSink<RunnerEvent<G>> for StdoutSink {
    fn emit(&mut self, event: RunnerEvent<G>) {
        match event {
            RunnerEvent::GameStarted { game_number, .. } => {
                println!("=== Game #{} ===\n", game_number + 1);
            }
            RunnerEvent::TurnStarted { turn_number, .. } => {
                println!("--- Turn #{} ---\n", turn_number + 1);
            }
            RunnerEvent::ActionApplied {
                turn,
                state,
                action,
                ..
            } => {
                println!("{:?} {}\n", turn, action);
                println!("{}", state.display(turn));
            }
            RunnerEvent::GameFinished { turn, outcome, .. } => {
                println!("{}", outcome.display(turn));
            }
            _ => {}
        }
    }
}
