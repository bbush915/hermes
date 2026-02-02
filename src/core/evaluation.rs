use crate::core::game::Game;

pub struct Evaluation<G: Game> {
    pub policy: Vec<PolicyItem<G>>,
    pub value: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PolicyItem<G: Game> {
    pub action: G::Action,
    pub prior: f32,
}
