use crate::core::{Evaluation, Game};

pub trait Evaluator<G: Game>: Clone {
    fn with_seed(self, seed: u64) -> Self;

    fn evaluate(&mut self, game: &G) -> Evaluation<G>;
}
