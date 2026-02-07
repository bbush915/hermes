use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

use hermes::{
    ActionEncoder, BoopActionEncoder, BoopStateEncoder, JsonSink, NeuralNetworkMctsPlayer,
    OnnxNeuralNetwork, RandomNeuralNetwork, Runner, SampleSink, StatisticsSink,
};

#[derive(Parser)]
#[command(name = "self-play")]
#[command(about = "Run self-play games and generate training data.")]
struct Args {
    #[arg(short, long, default_value = None)]
    output: Option<PathBuf>,

    #[arg(short, long, default_value_t = 1)]
    games: u64,

    #[arg(short, long, default_value_t = 100)]
    simulations: u32,
}

fn main() {
    let args = Args::parse();

    let state_encoder = BoopStateEncoder::new();
    let action_encoder = BoopActionEncoder::new();

    // let neural_network = RandomNeuralNetwork::new(action_encoder.size());

    let neural_network = OnnxNeuralNetwork::new(
        "/Users/bbush/Source/hermes/training/models/onnx/iter_0.onnx",
        state_encoder.clone(),
    )
    .expect("failed to load ONNX model");

    let player1 = NeuralNetworkMctsPlayer::new(
        args.simulations,
        state_encoder.clone(),
        action_encoder.clone(),
        neural_network,
    );

    let player2 = player1.clone();

    if args.output.is_none() {
        let sink = StatisticsSink::new();

        let mut runner = Runner::new(args.games, player1, player2, sink);

        runner.run();

        let StatisticsSink {
            player_wins,
            opponent_wins,
        } = runner.into_sink();

        println!(
            "Player wins: {}, Opponent wins: {}",
            player_wins, opponent_wins
        );
    } else {
        let file = File::create(&args.output.as_ref().unwrap()).expect(&format!(
            "Failed to create file: {:?}",
            args.output.as_ref().unwrap()
        ));

        let json_sink = JsonSink::new(file);
        let sample_sink = SampleSink::new(state_encoder, action_encoder, json_sink);

        let mut runner = Runner::new(args.games, player1, player2, sample_sink);

        runner.run();
    }
}
