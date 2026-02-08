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
            (Outcome::InProgress, _) => "Game is in progress.".to_string(),
            (Outcome::Win, Turn::PlayerOne) | (Outcome::Loss, Turn::PlayerTwo) => {
                "Player 1 wins!".to_string()
            }
            (Outcome::Win, Turn::PlayerTwo) | (Outcome::Loss, Turn::PlayerOne) => {
                "Player 2 wins!".to_string()
            }
            (Outcome::Draw, _) => "Game is a draw!".to_string(),
        }
    }
}
