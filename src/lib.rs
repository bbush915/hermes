mod core;
mod game;
mod neural_network;
mod player;
mod self_play;

pub use core::{Game, Outcome, Player, Runner, StdoutSink, Turn};
pub use game::{
    Boop, BoopActionEncoder, BoopStateEncoder, TicTacToe, TicTacToeActionEncoder,
    TicTacToeStateEncoder,
};
pub use neural_network::{ActionEncoder, RandomNeuralNetwork, StateEncoder};
pub use player::{ClassicMctsPlayer, MinimaxPlayer, NeuralNetworkMctsPlayer, RandomPlayer};
pub use self_play::{JsonSink, Sample, SampleSink};
