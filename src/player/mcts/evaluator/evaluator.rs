use crate::core::{Evaluation, Game};

pub trait Evaluator<G: Game>: Clone {
    fn evaluate(&mut self, game: &G) -> Evaluation<G>;
}
