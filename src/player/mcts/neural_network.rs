use crate::core::{Choice, Game, Player};
use crate::neural_network::{ActionEncoder, NeuralNetwork, StateEncoder};
use crate::player::mcts::evaluator::NeuralNetworkEvaluator;
use crate::player::mcts::expander::CompleteExpander;
use crate::player::mcts::mcts::{Mcts, MtcsOptions, SearchResult};
use crate::player::mcts::noise::DirichletNoise;
use crate::player::mcts::scorer::PuctScorer;
use crate::player::mcts::temperature::TemperatureSchedule;

#[derive(Clone)]
pub struct NeuralNetworkMctsPlayer<
    G: Game,
    SE: StateEncoder<G>,
    AE: ActionEncoder<G>,
    NN: NeuralNetwork,
> {
    mcts: Mcts<G, NeuralNetworkEvaluator<G, SE, AE, NN>, PuctScorer, CompleteExpander>,
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, NN: NeuralNetwork>
    NeuralNetworkMctsPlayer<G, SE, AE, NN>
{
    pub fn new(
        simulations: u32,
        state_encoder: SE,
        action_encoder: AE,
        neural_network: NN,
    ) -> Self {
        let options = MtcsOptions::new(
            simulations,
            NeuralNetworkEvaluator::new(state_encoder, action_encoder, neural_network),
            PuctScorer::new(),
            CompleteExpander::new(),
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

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, NN: NeuralNetwork> Player<G>
    for NeuralNetworkMctsPlayer<G, SE, AE, NN>
{
    fn name(&self) -> &'static str {
        "MCTS - Neural Network"
    }

    fn choose_action(&mut self, game: &G, turn_number: u32) -> Choice<G> {
        let SearchResult { action, evaluation } = self.mcts.search(game, turn_number);

        Choice {
            action,
            evaluation: Some(evaluation),
        }
    }
}
