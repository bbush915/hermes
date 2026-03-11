use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use serde::Serialize;

use hermes_engine::boop::{Boop, BoopActionEncoder, BoopStateEncoder};
use hermes_engine::{
    Choice, MinimaxPlayer, NeuralNetworkMctsPlayer, OnnxNeuralNetwork, Player, RandomPlayer,
    Runner, StatisticsRunnerEventSink, TemperatureSchedule,
};

// -- Player spec --

#[derive(Clone)]
enum PlayerSpec {
    Random,
    Minimax(usize),
    NeuralNetwork(PathBuf),
}

impl PlayerSpec {
    fn default_name(&self) -> String {
        match self {
            PlayerSpec::Random => "random".to_string(),
            PlayerSpec::Minimax(depth) => format!("minimax-{depth}"),
            PlayerSpec::NeuralNetwork(path) => path.display().to_string(),
        }
    }
}

impl FromStr for PlayerSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "random" {
            return Ok(PlayerSpec::Random);
        }
        if let Some(depth_str) = s.strip_prefix("minimax:") {
            let depth = depth_str.parse::<usize>().map_err(|e| e.to_string())?;
            return Ok(PlayerSpec::Minimax(depth));
        }
        Ok(PlayerSpec::NeuralNetwork(PathBuf::from(s)))
    }
}

// -- Concrete player enum (enables mixing player types at runtime) --

type BoopNnPlayer = NeuralNetworkMctsPlayer<
    Boop,
    BoopStateEncoder,
    BoopActionEncoder,
    OnnxNeuralNetwork<Boop, BoopStateEncoder>,
>;

enum BoopPlayer {
    Random(RandomPlayer),
    Minimax(MinimaxPlayer),
    NeuralNetwork(BoopNnPlayer),
}

impl Player<Boop> for BoopPlayer {
    fn name(&self) -> &str {
        match self {
            BoopPlayer::Random(p) => <RandomPlayer as Player<Boop>>::name(p),
            BoopPlayer::Minimax(p) => <MinimaxPlayer as Player<Boop>>::name(p),
            BoopPlayer::NeuralNetwork(p) => p.name(),
        }
    }

    fn choose_action(&mut self, game: &Boop, turn_number: u32) -> Choice<Boop> {
        match self {
            BoopPlayer::Random(p) => p.choose_action(game, turn_number),
            BoopPlayer::Minimax(p) => p.choose_action(game, turn_number),
            BoopPlayer::NeuralNetwork(p) => p.choose_action(game, turn_number),
        }
    }
}

fn build_player(spec: &PlayerSpec, simulations: u32) -> BoopPlayer {
    match spec {
        PlayerSpec::Random => BoopPlayer::Random(RandomPlayer::new()),
        PlayerSpec::Minimax(depth) => BoopPlayer::Minimax(MinimaxPlayer::new(*depth)),
        PlayerSpec::NeuralNetwork(path) => {
            let state_encoder = BoopStateEncoder::new();
            let action_encoder = BoopActionEncoder::new();
            let nn =
                OnnxNeuralNetwork::new(path, state_encoder).expect("failed to load ONNX model");
            BoopPlayer::NeuralNetwork(
                // No Dirichlet noise, constant temperature=0 (greedy) for fair evaluation.
                NeuralNetworkMctsPlayer::new(simulations, state_encoder, action_encoder, nn)
                    .with_temperature_schedule(TemperatureSchedule::Constant(0.0)),
            )
        }
    }
}

// -- ELO --

const ELO_K: f32 = 32.0;
const ELO_DEFAULT: f32 = 1000.0;

fn elo_expected(rating_a: f32, rating_b: f32) -> f32 {
    1.0 / (1.0 + 10f32.powf((rating_b - rating_a) / 400.0))
}

fn elo_update(rating_a: f32, rating_b: f32, score_a: f32) -> (f32, f32) {
    let expected_a = elo_expected(rating_a, rating_b);
    let expected_b = 1.0 - expected_a;
    let score_b = 1.0 - score_a;
    (
        rating_a + ELO_K * (score_a - expected_a),
        rating_b + ELO_K * (score_b - expected_b),
    )
}

// -- Output types --

#[derive(Serialize)]
struct PlayerResult {
    name: String,
    wins: u32,
    win_rate: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    elo_before: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    elo_after: Option<f32>,
}

#[derive(Serialize)]
struct EvalOutput {
    player1: PlayerResult,
    player2: PlayerResult,
    draws: u32,
    draw_rate: f32,
    total_games: u32,
}

// -- CLI --

#[derive(Parser)]
#[command(name = "evaluate")]
#[command(about = "Run evaluation games between two players.")]
struct Args {
    #[arg(short, long, default_value_t = 100)]
    games: u32,

    #[arg(long)]
    player1: PlayerSpec,

    #[arg(long)]
    player2: PlayerSpec,

    /// Display name for player 1, used as the ELO key. Defaults to the player spec string.
    #[arg(long)]
    player1_name: Option<String>,

    /// Display name for player 2, used as the ELO key. Defaults to the player spec string.
    #[arg(long)]
    player2_name: Option<String>,

    /// Number of MCTS simulations per move (only applies to neural network players).
    #[arg(short, long, default_value_t = 100)]
    simulations: u32,

    #[arg(short, long, default_value_t = 150)]
    max_turns: u32,

    #[arg(short, long, default_value_t = 1)]
    threads: usize,

    /// JSON file to read and update ELO ratings. Created if it does not exist.
    #[arg(long)]
    ratings: Option<PathBuf>,

    /// JSON file to write evaluation results.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let name1 = args
        .player1_name
        .unwrap_or_else(|| args.player1.default_name());
    let name2 = args
        .player2_name
        .unwrap_or_else(|| args.player2.default_name());

    let p1 = build_player(&args.player1, args.simulations);
    let p2 = build_player(&args.player2, args.simulations);

    let mut runner = Runner::new(args.games, p1, p2, StatisticsRunnerEventSink::new())
        .with_max_turns(args.max_turns)
        .with_threads(args.threads);

    runner.run();

    let stats = runner.sink();
    let total = stats.total_games;
    let p1_wins = stats.player_1_wins;
    let p2_wins = stats.player_2_wins;
    let draws = stats.draws;

    // ELO (only if --ratings provided)
    let mut ratings: HashMap<String, f32> = if let Some(path) = &args.ratings {
        if path.exists() {
            let json = fs::read_to_string(path).expect("failed to read ratings file");
            serde_json::from_str(&json).expect("failed to parse ratings file")
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    let elo_enabled = args.ratings.is_some();

    let (elo1_before, elo1_after, elo2_before, elo2_after) = if elo_enabled {
        let r1 = *ratings.get(&name1).unwrap_or(&ELO_DEFAULT);
        let r2 = *ratings.get(&name2).unwrap_or(&ELO_DEFAULT);
        let score1 = (p1_wins as f32 + 0.5 * draws as f32) / total as f32;
        let (r1_new, r2_new) = elo_update(r1, r2, score1);
        (Some(r1), Some(r1_new), Some(r2), Some(r2_new))
    } else {
        (None, None, None, None)
    };

    if elo_enabled {
        ratings.insert(name1.clone(), elo1_after.unwrap());
        ratings.insert(name2.clone(), elo2_after.unwrap());

        if let Some(path) = &args.ratings {
            let json = serde_json::to_string_pretty(&ratings).expect("failed to serialize ratings");
            fs::write(path, json).expect("failed to write ratings file");
        }

        println!("\nELO:");
        println!(
            "\t{}: {:.1} -> {:.1} ({:+.1})",
            name1,
            elo1_before.unwrap(),
            elo1_after.unwrap(),
            elo1_after.unwrap() - elo1_before.unwrap()
        );
        println!(
            "\t{}: {:.1} -> {:.1} ({:+.1})",
            name2,
            elo2_before.unwrap(),
            elo2_after.unwrap(),
            elo2_after.unwrap() - elo2_before.unwrap()
        );
    }

    if let Some(output_path) = &args.output {
        let output = EvalOutput {
            player1: PlayerResult {
                name: name1,
                wins: p1_wins,
                win_rate: p1_wins as f32 / total as f32,
                elo_before: elo1_before,
                elo_after: elo1_after,
            },
            player2: PlayerResult {
                name: name2,
                wins: p2_wins,
                win_rate: p2_wins as f32 / total as f32,
                elo_before: elo2_before,
                elo_after: elo2_after,
            },
            draws,
            draw_rate: draws as f32 / total as f32,
            total_games: total,
        };

        let json = serde_json::to_string_pretty(&output).expect("failed to serialize output");
        fs::write(output_path, json).expect("failed to write output file");
    }
}
