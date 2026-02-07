use hermes::{
    Boop, BoopActionEncoder, BoopStateEncoder, NeuralNetworkMctsPlayer, OnnxNeuralNetwork,
    RandomPlayer, Runner, StdoutSink, TicTacToe,
};

fn main() {
    // let player = RandomPlayer::new();
    // let opponent = RandomPlayer::new();

    let state_encoder = BoopStateEncoder::new();
    let action_encoder = BoopActionEncoder::new();

    // let neural_network = RandomNeuralNetwork::new(action_encoder.size());

    let neural_network = OnnxNeuralNetwork::new(
        "/Users/bbush/Source/hermes/training/models/onnx/iter_0.onnx",
        state_encoder.clone(),
    )
    .expect("failed to load ONNX model");

    let player1 = NeuralNetworkMctsPlayer::new(
        100,
        state_encoder.clone(),
        action_encoder.clone(),
        neural_network,
    );

    let player2 = player1.clone();

    let sink = StdoutSink::new();

    let mut runner = Runner::<Boop, _, _, _>::new(1, player1, player2, sink);

    runner.run();
}
