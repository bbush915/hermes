use crate::core::Game;
use crate::player::mcts::evaluator::{Evaluation, PolicyEntry};
use crate::player::mcts::expander::Expander;
use crate::player::mcts::mcts::Node;

#[derive(Debug)]
pub struct CompleteExpander;

impl CompleteExpander {
    pub fn new() -> Self {
        CompleteExpander
    }
}

impl<G: Game> Expander<G> for CompleteExpander {
    fn expand(&mut self, node: &mut Node<G>, evaluation: &Evaluation<G>) -> Vec<PolicyEntry<G>> {
        node.unexplored_actions.clear();

        evaluation.policy.clone()
    }
}
