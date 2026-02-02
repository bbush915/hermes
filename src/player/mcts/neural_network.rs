use crate::core::{Choice, Game, Player};
use crate::neural_network::{ActionEncoder, NeuralNetwork, StateEncoder};
use crate::player::mcts::evaluator::NeuralNetworkEvaluator;
use crate::player::mcts::expander::CompleteExpander;
use crate::player::mcts::mcts::{Mcts, SearchResult};
use crate::player::mcts::scorer::PuctScorer;

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
        Self {
            mcts: Mcts::new(
                simulations,
                NeuralNetworkEvaluator::new(state_encoder, action_encoder, neural_network),
                PuctScorer::new(),
                CompleteExpander::new(),
            ),
        }
    }
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, NN: NeuralNetwork> Player<G>
    for NeuralNetworkMctsPlayer<G, SE, AE, NN>
{
    fn name(&self) -> &str {
        "MCTS - Neural Network"
    }

    fn choose_action(&mut self, game: &G) -> Choice<G> {
        let SearchResult { action, evaluation } = self.mcts.search(game);

        Choice {
            action,
            evaluation: Some(evaluation),
        }
    }
}
