use std::fmt;

pub trait Game: Clone + fmt::Display {
    type Action: Copy + Eq + fmt::Display;
    type Checkpoint: Copy;

    fn new() -> Self;

    fn get_possible_actions(&self) -> Vec<Self::Action>;

    fn apply_action(&mut self, action: Self::Action) -> bool;

    fn outcome(&self) -> Outcome;

    fn create_checkpoint(&self) -> Self::Checkpoint;

    fn restore_checkpoint(&mut self, checkpoint: Self::Checkpoint);

    fn flip_perspective(&mut self);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    InProgress,
    Win,
    Loss,
    Draw,
}
