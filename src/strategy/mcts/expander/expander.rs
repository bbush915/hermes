use crate::core::Game;
use crate::strategy::mcts::evaluator::{Evaluation, PolicyEntry};
use crate::strategy::mcts::mcts::Node;

pub trait Expander<G: Game> {
    fn expand(&mut self, node: &mut Node<G>, evaluation: &Evaluation<G>) -> Vec<PolicyEntry<G>>;
}
