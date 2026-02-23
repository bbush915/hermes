use crate::core::Game;

pub trait StateEncoder<G: Game>: Copy {
    fn shape(&self) -> Vec<usize>;

    fn encode(&self, state: &G) -> Vec<f32>;
}
