use crate::core::{Game, Player};
use crate::strategy::mcts::evaluator::RolloutEvaluator;
use crate::strategy::mcts::expander::RandomExpander;
use crate::strategy::mcts::mcts::Mcts;
use crate::strategy::mcts::scorer::Ucb1Scorer;

pub struct ClassicMctsPlayer<G: Game> {
    mcts: Mcts<G, RolloutEvaluator, Ucb1Scorer, RandomExpander>,
}

impl<G: Game> ClassicMctsPlayer<G> {
    pub fn new(simulations: u32) -> Self {
        Self {
            mcts: Mcts::new(
                simulations,
                RolloutEvaluator::new(),
                Ucb1Scorer::new(),
                RandomExpander::new(),
            ),
        }
    }
}

impl<G: Game> Player<G> for ClassicMctsPlayer<G> {
    fn name(&self) -> &str {
        "MCTS - Classic"
    }

    fn choose_action(&mut self, game: &G) -> G::Action {
        self.mcts.search(game)
    }
}
