use crate::core::Game;

pub trait ActionEncoder<G: Game>: Copy {
    const ACTION_COUNT: usize;

    fn size(&self) -> usize {
        Self::ACTION_COUNT
    }

    fn encode(&self, action: &G::Action) -> usize;
    fn decode(&self, action_id: usize) -> G::Action;
}
