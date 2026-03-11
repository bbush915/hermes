use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{SeedableRng, rng};

use crate::core::{Choice, Game, Player};

pub struct RandomPlayer {
    rng: StdRng,
}

impl Clone for RandomPlayer {
    fn clone(&self) -> Self {
        Self {
            rng: StdRng::from_rng(&mut rng()),
        }
    }
}

impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_rng(&mut rng()),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }
}

impl Default for RandomPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game> Player<G> for RandomPlayer {
    fn name(&self) -> &'static str {
        "Random"
    }

    fn choose_action(&mut self, game: &G, _turn_number: u32) -> Choice<G> {
        let actions = game.get_possible_actions();

        match actions.choose(&mut self.rng) {
            Some(action) => Choice {
                evaluation: None,
                action: *action,
            },
            None => panic!("no legal actions available"),
        }
    }
}
