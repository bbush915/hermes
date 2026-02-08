use crate::core::event::EventSink;
use crate::core::game::Game;
use crate::core::game::Outcome;
use crate::core::runner::runner::{RunnerEvent, RunnerEventContext, RunnerEventKind};
use crate::core::turn::Turn;

#[derive(Clone, Copy, Default)]
pub struct StatisticsRunnerEventSink {
    pub total_games: u32,
    pub player_1_wins: u32,
    pub player_2_wins: u32,
    pub draws: u32,
}

impl StatisticsRunnerEventSink {
    pub fn new() -> Self {
        StatisticsRunnerEventSink {
            total_games: 0,
            player_1_wins: 0,
            player_2_wins: 0,
            draws: 0,
        }
    }
}

impl<G: Game> EventSink<RunnerEvent<G>> for StatisticsRunnerEventSink {
    fn emit(&mut self, event: RunnerEvent<G>) {
        let RunnerEvent { kind, context, .. } = event;

        match kind {
            RunnerEventKind::GameFinished { outcome } => {
                let RunnerEventContext { turn, .. } = context.expect("event is missing context");

                self.total_games += 1;

                match (outcome, turn) {
                    (Outcome::Win, Turn::PlayerOne) | (Outcome::Loss, Turn::PlayerTwo) => {
                        self.player_1_wins += 1;
                    }
                    (Outcome::Win, Turn::PlayerTwo) | (Outcome::Loss, Turn::PlayerOne) => {
                        self.player_2_wins += 1;
                    }
                    (Outcome::Draw, _) => self.draws += 1,
                    _ => {}
                }
            }
            RunnerEventKind::RunnerFinished => {
                println!("Statistics:");
                println!("\tTotal Games: {}", self.total_games);
                println!(
                    "\tPlayer 1 Wins: {} ({:.2}%)",
                    self.player_1_wins,
                    self.player_1_wins as f32 / self.total_games as f32 * 100.0
                );
                println!(
                    "\tPlayer 2 Wins: {} ({:.2}%)",
                    self.player_2_wins,
                    self.player_2_wins as f32 / self.total_games as f32 * 100.0
                );
                println!(
                    "\tDraws: {} ({:.2}%)",
                    self.draws,
                    self.draws as f32 / self.total_games as f32 * 100.0
                );
            }
            _ => {}
        }
    }
}
