use crate::core::{Evaluation, Game, PolicyItem};
use crate::player::mcts::expander::Expander;
use crate::player::mcts::tree::Node;

#[derive(Clone, Debug, Default)]
pub struct CompleteExpander;

impl CompleteExpander {
    pub fn new() -> Self {
        CompleteExpander
    }
}

impl<G: Game> Expander<G> for CompleteExpander {
    fn with_seed(self, _seed: u64) -> Self {
        self
    }

    fn expand(&mut self, node: &mut Node<G>, evaluation: &Evaluation<G>) -> Vec<PolicyItem<G>> {
        node.unexplored_actions.clear();

        evaluation.policy.clone()
    }
}
