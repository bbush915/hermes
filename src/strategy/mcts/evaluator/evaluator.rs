use crate::core::Game;

pub trait Evaluator<G: Game> {
    fn evaluate(&mut self, game: &G) -> Evaluation<G>;
}

pub struct Evaluation<G: Game> {
    pub policy: Vec<PolicyEntry<G>>,
    pub value: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PolicyEntry<G: Game> {
    pub action: G::Action,
    pub prior: f32,
}
