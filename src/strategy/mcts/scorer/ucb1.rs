use std::f32;

use crate::core::Game;
use crate::strategy::mcts::mcts::Node;
use crate::strategy::mcts::scorer::scorer::Scorer;

pub struct Ucb1Scorer {
    c: f32,
}

impl Ucb1Scorer {
    pub fn new() -> Self {
        Self {
            c: f32::consts::SQRT_2,
        }
    }
}

impl<G: Game> Scorer<G> for Ucb1Scorer {
    fn score(&self, parent: &Node<G>, child: &Node<G>) -> f32 {
        if child.visits == 0 {
            return f32::INFINITY;
        }

        let parent_visits = parent.visits as f32;
        let child_visits = child.visits as f32;

        let exploitation = child.total_value / child_visits;
        let exploration = self.c * ((parent_visits.ln() / child_visits).sqrt());

        exploitation + exploration
    }
}
