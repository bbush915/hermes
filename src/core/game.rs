use std::fmt::Display;

pub trait Game: Clone + Display {
    type Action: Copy + Eq;
    type Checkpoint: Copy;

    fn outcome(&self) -> Outcome;

    fn get_possible_actions(&self) -> Vec<Self::Action>;

    fn apply_action(&mut self, action: Self::Action) -> bool;

    fn create_checkpoint(&self) -> Self::Checkpoint;

    fn restore_checkpoint(&mut self, checkpoint: Self::Checkpoint);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    InProgress,
    Win,
    Loss,
    Draw,
}
