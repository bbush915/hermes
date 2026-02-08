use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};

use crate::neural_network::neural_network::{NeuralNetwork, Prediction};

#[derive(Clone)]
pub struct RandomNeuralNetwork {
    rng: StdRng,

    policy_size: usize,
}

impl RandomNeuralNetwork {
    pub fn new(policy_size: usize) -> Self {
        Self {
            rng: StdRng::from_entropy(),

            policy_size,
        }
    }
}

impl NeuralNetwork for RandomNeuralNetwork {
    fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }

    fn predict(&mut self, _input: &[f32]) -> Prediction {
        let distribution = Normal::new(0.0, 1.0).unwrap();

        let policy_logits = std::iter::from_fn(|| Some(distribution.sample(&mut self.rng)))
            .take(self.policy_size)
            .collect();

        let value = self.rng.r#gen::<f32>() * 2.0 - 1.0;

        Prediction {
            policy_logits,
            value,
        }
    }
}
