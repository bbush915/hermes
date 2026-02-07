mod core;
mod game;
mod neural_network;
mod player;
mod self_play;

pub use core::{
    EventSink, Game, NullSink, Outcome, Player, Runner, RunnerEvent, StatisticsSink, StdoutSink,
    Turn,
};
pub use game::{
    Boop, BoopActionEncoder, BoopStateEncoder, TicTacToe, TicTacToeActionEncoder,
    TicTacToeStateEncoder,
};
pub use neural_network::{
    ActionEncoder, NeuralNetwork, OnnxNeuralNetwork, RandomNeuralNetwork, StateEncoder,
};
pub use player::{ClassicMctsPlayer, MinimaxPlayer, NeuralNetworkMctsPlayer, RandomPlayer};
pub use self_play::{JsonSink, Sample, SampleSink};

#[test]
fn test_untrained_network_outputs() {
    let mut nn = OnnxNeuralNetwork::new(
        "/Users/bbush/Source/hermes/training/models/onnx/iter_0.onnx",
        BoopStateEncoder::new(),
    )
    .unwrap();

    // Test with zero input
    let zero_input = vec![0.0; 360];
    let (policy, value) = nn.forward(&zero_input);

    // Check basic properties
    assert_eq!(policy.len(), 188, "Policy should have 188 elements");
    assert!(value >= -1.0 && value <= 1.0, "Value should be in [-1, 1]");
    assert!(policy.iter().all(|&p| p >= 0.0), "All probabilities >= 0");

    let policy_sum: f32 = policy.iter().sum();
    println!("Policy sum: {}", policy_sum);
    assert!((policy_sum - 1.0).abs() < 0.01, "Policy should sum to ~1.0");

    // Untrained network should give roughly uniform policy
    let avg_prob = 1.0 / 188.0;
    let max_prob = policy.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    println!("Max prob: {}, Average: {}", max_prob, avg_prob);

    // Value should be near 0 (neutral)
    println!("Value prediction: {}", value);
    assert!(value.abs() < 0.5, "Untrained value should be near 0");
}

#[test]
fn test_with_softmax() {
    let mut nn = OnnxNeuralNetwork::new(
        "/Users/bbush/Source/hermes/training/models/onnx/iter_0.onnx",
        BoopStateEncoder::new(),
    )
    .unwrap();

    let input = vec![0.0; 360];
    let (raw_policy, value) = nn.forward(&input);

    println!("Raw policy (first 10): {:?}", &raw_policy[..10]);
    println!(
        "Min: {}, Max: {}",
        raw_policy.iter().copied().fold(f32::INFINITY, f32::min),
        raw_policy.iter().copied().fold(f32::NEG_INFINITY, f32::max)
    );

    // Apply softmax
    let policy = softmax(&raw_policy);

    println!("\nAfter softmax (first 10): {:?}", &policy[..10]);
    println!("Sum: {}", policy.iter().sum::<f32>());

    assert!(policy.iter().all(|&p| p >= 0.0));
    assert!((policy.iter().sum::<f32>() - 1.0).abs() < 0.01);
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&x| (x - max_logit).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}
