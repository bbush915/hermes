use std::fmt;

use crate::core::turn::Turn;

pub trait Game: Clone + fmt::Display {
    type Action: Copy + Eq + fmt::Display;
    type Checkpoint: Copy;

    fn new() -> Self;

    fn get_possible_actions(&self) -> Vec<Self::Action>;

    fn apply_action(&mut self, action: Self::Action) -> bool;

    fn end_turn(&mut self);

    fn outcome(&self) -> Outcome;

    fn create_checkpoint(&self) -> Self::Checkpoint;

    fn restore_checkpoint(&mut self, checkpoint: Self::Checkpoint);

    fn display(&self, turn: Turn) -> String;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    InProgress,
    Win,
    Loss,
    Draw,
}

impl Outcome {
    pub fn display(&self, turn: Turn) -> String {
        match (self, turn) {
            (Outcome::Win, Turn::Player) => "Player wins!".to_string(),
            (Outcome::Win, Turn::Opponent) => "Opponent wins!".to_string(),
            (Outcome::Loss, Turn::Player) => "Player loses!".to_string(),
            (Outcome::Loss, Turn::Opponent) => "Opponent loses!".to_string(),
            (Outcome::Draw, _) => "Game is a draw!".to_string(),
            (Outcome::InProgress, _) => "Game is in progress.".to_string(),
        }
    }
}
