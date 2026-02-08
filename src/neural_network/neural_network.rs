pub trait NeuralNetwork: Clone {
    fn with_seed(self, seed: u64) -> Self;

    fn forward(&mut self, input: &[f32]) -> (Vec<f32>, f32);
}
