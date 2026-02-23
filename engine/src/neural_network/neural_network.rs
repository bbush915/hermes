pub trait NeuralNetwork: Clone {
    fn with_seed(self, seed: u64) -> Self;

    fn predict(&mut self, input: &[f32]) -> Prediction;
}

pub struct Prediction {
    pub policy_logits: Vec<f32>,
    pub value: f32,
}
