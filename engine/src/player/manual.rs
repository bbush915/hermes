use crate::core::{Choice, Game, Player};

pub struct ManualPlayer<G: Game> {
    queued_action: Option<G::Action>,
}

impl<G: Game> ManualPlayer<G> {
    pub fn new() -> Self {
        Self {
            queued_action: None,
        }
    }

    pub fn queue_action(&mut self, action: G::Action) {
        self.queued_action = Some(action);
    }
}

impl<G: Game> Default for ManualPlayer<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game> Player<G> for ManualPlayer<G> {
    fn name(&self) -> &'static str {
        "Manual"
    }

    fn choose_action(&mut self, _game: &G, _turn_number: u32) -> Choice<G> {
        let action = self
            .queued_action
            .take()
            .expect("no action queued for manual player");

        Choice {
            evaluation: None,
            action,
        }
    }
}
