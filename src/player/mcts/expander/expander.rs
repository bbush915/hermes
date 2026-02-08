use crate::core::{Evaluation, Game, PolicyItem};
use crate::player::mcts::tree::Node;

pub trait Expander<G: Game>: Clone {
    fn with_seed(self, seed: u64) -> Self;

    fn expand(&mut self, node: &mut Node<G>, evaluation: &Evaluation<G>) -> Vec<PolicyItem<G>>;
}
