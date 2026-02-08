use crate::core::evaluation::Evaluation;
use crate::core::game::Game;

pub trait Player<G: Game>: Clone {
    fn name(&self) -> &str;

    fn choose_action(&mut self, game: &G, turn_number: u32) -> Choice<G>;
}

pub struct Choice<G: Game> {
    pub evaluation: Option<Evaluation<G>>,
    pub action: G::Action,
}
