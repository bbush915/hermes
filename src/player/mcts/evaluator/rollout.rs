use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::core::{Evaluation, Game, Outcome, PolicyItem};
use crate::player::mcts::evaluator::Evaluator;

#[derive(Clone, Debug)]
pub struct RolloutEvaluator {
    rng: StdRng,
}

impl RolloutEvaluator {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    fn rollout<G: Game>(&mut self, game: &G) -> f32 {
        let mut game = game.clone();

        loop {
            let actions = game.get_possible_actions();

            if actions.is_empty() {
                return match game.outcome() {
                    Outcome::Win => 1.0,
                    Outcome::Loss => -1.0,
                    Outcome::Draw => 0.0,
                    Outcome::InProgress => unreachable!(),
                };
            }

            let Some(&action) = actions.choose(&mut self.rng) else {
                panic!("no legal actions available")
            };

            game.apply_action(action);
        }
    }
}

impl Default for RolloutEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game> Evaluator<G> for RolloutEvaluator {
    fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }

    fn evaluate(&mut self, game: &G) -> Evaluation<G> {
        if game.outcome() != Outcome::InProgress {
            let value = match game.outcome() {
                Outcome::Win => 1.0,
                Outcome::Loss => -1.0,
                Outcome::Draw => 0.0,
                Outcome::InProgress => unreachable!(),
            };

            return Evaluation {
                policy: vec![],
                value,
            };
        }

        let actions = game.get_possible_actions();

        let uniform_prior = 1.0 / actions.len() as f32;

        let policy = actions
            .iter()
            .map(|&action| PolicyItem {
                action,
                prior: uniform_prior,
            })
            .collect();

        let value = self.rollout(game);

        Evaluation { policy, value }
    }
}
