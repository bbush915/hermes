use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};

use crate::core::{Choice, Game, Player};

#[derive(Clone)]
pub struct RandomPlayer {
    rng: StdRng,
}

impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_os_rng(),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }
}

impl<G: Game> Player<G> for RandomPlayer {
    fn name(&self) -> &str {
        "Random"
    }

    fn choose_action(&mut self, game: &G) -> Choice<G> {
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
