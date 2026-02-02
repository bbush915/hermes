use crate::core::Game;

pub trait ActionEncoder<G: Game>: Clone {
    const ACTION_COUNT: usize;

    fn size(&self) -> usize {
        Self::ACTION_COUNT
    }

    fn encode(&self, action: &G::Action) -> usize;
    fn decode(&self, action_id: usize) -> G::Action;

    fn legal_action_mask(&self, game: &G) -> Vec<bool> {
        let mut mask = vec![false; Self::ACTION_COUNT];

        for action in game.get_possible_actions() {
            let action_id = self.encode(&action);

            mask[action_id] = true;
        }

        mask
    }
}
