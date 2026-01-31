use crate::core::Game;
use crate::player::mcts::mcts::Node;
use crate::player::mcts::scorer::scorer::Scorer;

pub struct PuctScorer {
    pub c_puct: f32,
}

impl<G: Game> Scorer<G> for PuctScorer {
    fn score(&self, parent: &Node<G>, child: &Node<G>) -> f32 {
        let exploitation = if child.visits == 0 {
            0.0
        } else {
            child.total_value / child.visits as f32
        };

        let child_visits = child.visits as f32;
        let parent_visits = parent.visits as f32;

        let exploration = self.c_puct * child.prior * (parent_visits.sqrt() / (1.0 + child_visits));

        exploitation + exploration
    }
}
