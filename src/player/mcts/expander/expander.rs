use crate::core::{Evaluation, Game, PolicyItem};
use crate::player::mcts::mcts::Node;

pub trait Expander<G: Game>: Clone {
    fn expand(&mut self, node: &mut Node<G>, evaluation: &Evaluation<G>) -> Vec<PolicyItem<G>>;
}
