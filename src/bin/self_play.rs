#![allow(unused)]

use std::time::Instant;

use clap::Parser;
use hermes::{
    Boop, ClassicMctsPlayer, Game, MinimaxPlayer, Outcome, Player, RandomPlayer, TicTacToe, Turn,
};

#[derive(Parser, Debug)]
#[command(name = "self_play")]
#[command(about = "Run self-play games between AI strategies", long_about = None)]
struct Args {
    /// Number of games to play
    #[arg(short, long, default_value_t = 1)]
    num_games: usize,
}

fn play_random_game() -> (Turn, usize) {
    let mut game = TicTacToe::new();

    let mut turn = Turn::Player;
    let mut turns = 0;

    let mut player1 = MinimaxPlayer::new(10);
    let mut player2 = MinimaxPlayer::new(10);

    loop {
        let action = if turn == Turn::Player {
            player1.choose_action(&game)
        } else {
            player2.choose_action(&game)
        };

        let turn_ended = game.apply_action(action);

        if turn_ended {
            turn = turn.flip();
            turns += 1;
        }

        if game.outcome() != Outcome::InProgress {
            break;
        }
    }

    (turn, turns)
}

fn main() {
    let args = Args::parse();

    let mut player1_wins = 0;
    let mut player2_wins = 0;
    let mut total_turns = 0;

    println!("Running {} game(s)...", args.num_games);

    let start_time = Instant::now();

    for i in 0..args.num_games {
        let (winner, turns) = play_random_game();
        total_turns += turns;

        if winner == Turn::Player {
            player1_wins += 1;
        } else {
            player2_wins += 1;
        }

        if args.num_games == 1 {
            println!(
                "Game finished in {} turns. Winner: Player {}",
                turns,
                match winner {
                    Turn::Player => 1,
                    Turn::Opponent => 2,
                }
            );
        } else if (i + 1) % 100 == 0 || i + 1 == args.num_games {
            let elapsed = start_time.elapsed();
            println!(
                "Completed {}/{} games ({:.2}s, {:.2} games/sec)",
                i + 1,
                args.num_games,
                elapsed.as_secs_f64(),
                (i + 1) as f64 / elapsed.as_secs_f64()
            );
        }
    }

    let total_time = start_time.elapsed();

    if args.num_games > 1 {
        println!("\n=== Results ===");
        println!("Total games: {}", args.num_games);
        println!(
            "Player 1 (MCTS): {} wins ({:.1}%)",
            player1_wins,
            (player1_wins as f64 / args.num_games as f64) * 100.0
        );
        println!(
            "Player 2 (Random): {} wins ({:.1}%)",
            player2_wins,
            (player2_wins as f64 / args.num_games as f64) * 100.0
        );
        println!(
            "Average turns per game: {:.1}",
            total_turns as f64 / args.num_games as f64
        );
        println!(
            "Total time: {:.2}s ({:.2} games/sec, {:.2}ms/game)",
            total_time.as_secs_f64(),
            args.num_games as f64 / total_time.as_secs_f64(),
            total_time.as_millis() as f64 / args.num_games as f64
        );
    }
}
