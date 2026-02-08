use crate::core::Game;
use crate::player::mcts::tree::Node;

pub trait Scorer<G: Game>: Clone {
    fn score(&self, parent: &Node<G>, child: &Node<G>) -> f32;
}
