pub trait NeuralNetwork: Clone {
    fn forward(&mut self, input: &[f32]) -> (Vec<f32>, f32);
}
