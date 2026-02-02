use hermes::{Boop, RandomPlayer, Runner, StdoutSink};

fn main() {
    let player = RandomPlayer::new();
    let opponent = RandomPlayer::new();

    let sink = StdoutSink::new();

    let mut runner = Runner::<Boop, _, _, _>::new(1, player, opponent, sink);

    runner.run();
}
