use hermes::{Boop, RandomPlayer, Runner, StdoutRunnerEventSink};

fn main() {
    let player_1 = RandomPlayer::new();
    let player_2 = RandomPlayer::new();

    let sink = StdoutRunnerEventSink::new();

    let mut runner = Runner::<Boop, _, _, _>::new(1, player_1, player_2, sink);

    runner.run();
}
