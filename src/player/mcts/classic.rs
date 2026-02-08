use crate::core::{Choice, Game, Player};
use crate::player::mcts::evaluator::RolloutEvaluator;
use crate::player::mcts::expander::RandomExpander;
use crate::player::mcts::mcts::{Mcts, MtcsOptions, SearchResult};
use crate::player::mcts::noise::DirichletNoise;
use crate::player::mcts::scorer::Ucb1Scorer;
use crate::player::mcts::temperature::TemperatureSchedule;

#[derive(Clone)]
pub struct ClassicMctsPlayer<G: Game> {
    mcts: Mcts<G, RolloutEvaluator, Ucb1Scorer, RandomExpander>,
}

impl<G: Game> ClassicMctsPlayer<G> {
    pub fn new(simulations: u32) -> Self {
        let options = MtcsOptions::new(
            simulations,
            RolloutEvaluator::new(),
            Ucb1Scorer::new(),
            RandomExpander::new(),
        );

        Self {
            mcts: Mcts::new(options),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.mcts = self.mcts.with_seed(seed);

        self
    }

    pub fn with_dirichlet_noise(mut self, dirichlet_noise: DirichletNoise) -> Self {
        self.mcts = self.mcts.with_dirichlet_noise(dirichlet_noise);

        self
    }

    pub fn with_temperature_schedule(mut self, temperature_schedule: TemperatureSchedule) -> Self {
        self.mcts = self.mcts.with_temperature_schedule(temperature_schedule);

        self
    }
}

impl<G: Game> Player<G> for ClassicMctsPlayer<G> {
    fn name(&self) -> &'static str {
        "MCTS - Classic"
    }

    fn choose_action(&mut self, game: &G, turn_number: u32) -> Choice<G> {
        let SearchResult { action, evaluation } = self.mcts.search(game, turn_number);

        Choice {
            evaluation: Some(evaluation),
            action,
        }
    }
}
