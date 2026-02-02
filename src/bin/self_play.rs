use hermes::{
    ActionEncoder, BoopActionEncoder, BoopStateEncoder, JsonSink, NeuralNetworkMctsPlayer,
    RandomNeuralNetwork, Runner, SampleSink,
};
use std::fs::File;

fn main() {
    let state_encoder = BoopStateEncoder::new();
    let action_encoder = BoopActionEncoder::new();

    let neural_network = RandomNeuralNetwork::new(action_encoder.size());

    let player1 = NeuralNetworkMctsPlayer::new(
        10,
        state_encoder.clone(),
        action_encoder.clone(),
        neural_network,
    );

    let player2 = player1.clone();

    let file = File::create("test.jsonl").expect("Failed to create file");

    let json_sink = JsonSink::new(file);
    let sample_sink = SampleSink::new(state_encoder, action_encoder, json_sink);

    let mut runner = Runner::new(1, player1, player2, sample_sink);

    runner.run();
}
