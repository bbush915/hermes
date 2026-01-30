#![allow(unused)]

mod core;
mod game;
mod strategy;

pub use core::{Game, Outcome, Player, Turn};
pub use game::{Boop, TicTacToe};
pub use strategy::{ClassicMctsPlayer, MinimaxPlayer, RandomPlayer};
