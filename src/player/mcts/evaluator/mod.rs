#[allow(clippy::module_inception)]
mod evaluator;
mod neural_network;
mod rollout;

pub use evaluator::Evaluator;
pub use neural_network::NeuralNetworkEvaluator;
pub use rollout::RolloutEvaluator;
