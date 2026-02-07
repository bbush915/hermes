use crate::core::{EventSink, Game, Outcome, RunnerEvent, Turn};

pub struct StatisticsSink {
    pub player_wins: u64,
    pub opponent_wins: u64,
}

impl StatisticsSink {
    pub fn new() -> Self {
        StatisticsSink {
            player_wins: 0,
            opponent_wins: 0,
        }
    }
}

impl<G: Game> EventSink<RunnerEvent<G>> for StatisticsSink {
    fn emit(&mut self, event: RunnerEvent<G>) {
        match event {
            RunnerEvent::GameFinished { outcome, turn, .. } => match (outcome, turn) {
                (Outcome::Win, Turn::Player) => self.player_wins += 1,
                (Outcome::Win, Turn::Opponent) => self.opponent_wins += 1,
                (Outcome::Loss, Turn::Player) => self.opponent_wins += 1,
                (Outcome::Loss, Turn::Opponent) => self.player_wins += 1,
                _ => {} // Draws or in progress
            },
            _ => {}
        }
    }
}
