use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::neural_network::neural_network::NeuralNetwork;

#[derive(Clone)]
pub struct RandomNeuralNetwork {
    rng: StdRng,

    policy_size: usize,
}

impl RandomNeuralNetwork {
    pub fn new(policy_size: usize) -> Self {
        Self {
            rng: StdRng::from_os_rng(),

            policy_size,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self
    }
}

impl NeuralNetwork for RandomNeuralNetwork {
    fn forward(&mut self, _input: &[f32]) -> (Vec<f32>, f32) {
        let rng = &mut self.rng;

        let mut logits = vec![0.0; self.policy_size];

        for logit in &mut logits {
            *logit = rng.random::<f32>();
        }

        let value = rng.random::<f32>() * 2.0 - 1.0;

        (logits, value)
    }
}
