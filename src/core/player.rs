use crate::core::Game;

pub trait Player<G: Game> {
    fn name(&self) -> &str;

    fn choose_action(&mut self, game: &G) -> G::Action;
}
