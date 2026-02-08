use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

use hermes::{
    BoopActionEncoder, BoopStateEncoder, DirichletNoise, JsonSampleSink, NeuralNetworkMctsPlayer,
    OnnxNeuralNetwork, Runner, SampleRunnerEventSink, StatisticsRunnerEventSink,
    TemperatureSchedule,
};

#[derive(Parser)]
#[command(name = "self-play")]
#[command(about = "Run self-play games and generate training data.")]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    games: u32,

    #[arg(short, long)]
    model: PathBuf,

    #[arg(short, long, default_value_t = 100)]
    simulations: u32,

    #[arg(short, long, default_value_t = 150)]
    max_turns: u32,

    #[arg(short, long, default_value_t = false)]
    use_symmetries: bool,

    #[arg(short, long, default_value = None)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let state_encoder = BoopStateEncoder::new();
    let action_encoder = BoopActionEncoder::new();

    let neural_network =
        OnnxNeuralNetwork::new(args.model, state_encoder).expect("failed to load onnx model");

    let player_1 = NeuralNetworkMctsPlayer::new(
        args.simulations,
        state_encoder,
        action_encoder,
        neural_network,
    )
    .with_dirichlet_noise(DirichletNoise {
        alpha: 0.3,
        epsilon: 0.25,
    })
    .with_temperature_schedule(TemperatureSchedule::Step {
        threshold: 30,
        hi: 1.0,
        lo: 0.0,
    });

    let player_2 = player_1.clone();

    if let Some(path) = &args.output {
        let file = File::create(path).expect("failed to create output file");

        let json_sink = JsonSampleSink::new(file);

        let sample_sink = SampleRunnerEventSink::new(
            state_encoder,
            action_encoder,
            args.use_symmetries,
            json_sink,
        );

        let mut runner =
            Runner::new(args.games, player_1, player_2, sample_sink).with_max_turns(args.max_turns);

        runner.run();
    } else {
        let statistics_sink = StatisticsRunnerEventSink::new();

        let mut runner = Runner::new(args.games, player_1, player_2, statistics_sink)
            .with_max_turns(args.max_turns);

        runner.run();
    }
}
