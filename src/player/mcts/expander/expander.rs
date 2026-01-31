use crate::core::Game;
use crate::player::mcts::evaluator::{Evaluation, PolicyEntry};
use crate::player::mcts::mcts::Node;

pub trait Expander<G: Game> {
    fn expand(&mut self, node: &mut Node<G>, evaluation: &Evaluation<G>) -> Vec<PolicyEntry<G>>;
}
