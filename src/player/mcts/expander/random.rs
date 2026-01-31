use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::core::Game;
use crate::player::mcts::evaluator::{Evaluation, PolicyEntry};
use crate::player::mcts::expander::Expander;
use crate::player::mcts::mcts::Node;

#[derive(Debug)]
pub struct RandomExpander {
    rng: StdRng,
}

impl RandomExpander {
    pub fn new() -> Self {
        RandomExpander {
            rng: StdRng::from_os_rng(),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }
}

impl<G: Game> Expander<G> for RandomExpander {
    fn expand(&mut self, node: &mut Node<G>, _evaluation: &Evaluation<G>) -> Vec<PolicyEntry<G>> {
        if node.unexplored_actions.is_empty() {
            return vec![];
        }

        let action_index = self.rng.random_range(0..node.unexplored_actions.len());
        let action = node.unexplored_actions[action_index];

        node.unexplored_actions.swap_remove(action_index);

        vec![PolicyEntry { action, prior: 1.0 }]
    }
}
