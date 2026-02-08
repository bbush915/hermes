use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::neural_network::neural_network::NeuralNetwork;

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

    fn forward(&mut self, _input: &[f32]) -> (Vec<f32>, f32) {
        let rng = &mut self.rng;

        let mut logits = vec![0.0; self.policy_size];

        for logit in &mut logits {
            *logit = rng.r#gen::<f32>();
        }

        let value = rng.r#gen::<f32>() * 2.0 - 1.0;

        (logits, value)
    }
}
