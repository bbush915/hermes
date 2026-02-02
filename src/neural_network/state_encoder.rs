use crate::core::Game;

pub trait StateEncoder<G: Game>: Clone {
    fn size(&self) -> (usize, usize, usize);

    fn encode(&self, state: &G) -> Vec<f32>;
}
