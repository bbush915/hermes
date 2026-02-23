use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::core::{Evaluation, Game, PolicyItem};
use crate::player::mcts::expander::Expander;
use crate::player::mcts::tree::Node;

#[derive(Clone, Debug)]
pub struct RandomExpander {
    rng: StdRng,
}

impl RandomExpander {
    pub fn new() -> Self {
        RandomExpander {
            rng: StdRng::from_entropy(),
        }
    }
}

impl Default for RandomExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game> Expander<G> for RandomExpander {
    fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }

    fn expand(&mut self, node: &mut Node<G>, _evaluation: &Evaluation<G>) -> Vec<PolicyItem<G>> {
        if node.unexplored_actions.is_empty() {
            return vec![];
        }

        let action_index = self.rng.gen_range(0..node.unexplored_actions.len());
        let action = node.unexplored_actions[action_index];

        node.unexplored_actions.swap_remove(action_index);

        vec![PolicyItem { action, prior: 1.0 }]
    }
}
