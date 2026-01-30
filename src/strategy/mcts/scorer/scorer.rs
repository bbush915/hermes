use crate::core::Game;
use crate::strategy::mcts::mcts::Node;

pub trait Scorer<G: Game> {
    fn score(&self, parent: &Node<G>, child: &Node<G>) -> f32;
}
