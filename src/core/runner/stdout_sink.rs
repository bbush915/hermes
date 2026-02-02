use crate::core::{EventSink, Game, Outcome, RunnerEvent, Turn};

pub struct StdoutSink;

impl StdoutSink {
    pub fn new() -> Self {
        StdoutSink
    }
}

impl<G: Game> EventSink<RunnerEvent<G>> for StdoutSink {
    fn emit(&mut self, event: RunnerEvent<G>) {
        match event {
            RunnerEvent::GameStarted { game_id } => {
                println!("=== Game #{} ===", game_id + 1);
            }
            RunnerEvent::ActionApplied {
                state,
                turn,
                action,
            } => {
                println!("{:?} {}", turn, action);
                println!("{}", state);
            }
            RunnerEvent::GameFinished { turn, outcome, .. } => {
                let outcome = if turn == Turn::Opponent {
                    outcome
                } else {
                    match outcome {
                        Outcome::Win => Outcome::Loss,
                        Outcome::Loss => Outcome::Win,
                        Outcome::Draw => Outcome::Draw,
                        Outcome::InProgress => unreachable!(),
                    }
                };

                match outcome {
                    Outcome::Win => println!("\nPlayer wins!\n"),
                    Outcome::Loss => println!("\nPlayer loses!\n"),
                    Outcome::Draw => println!("\nGame is a draw!\n"),
                    _ => unreachable!(),
                }

                println!("===============");
            }
            _ => {}
        }
    }
}
