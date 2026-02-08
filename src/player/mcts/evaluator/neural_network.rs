use std::f32;
use std::marker::PhantomData;

use crate::core::{Evaluation, Game, PolicyItem};
use crate::neural_network::{ActionEncoder, NeuralNetwork, Prediction, StateEncoder};
use crate::player::mcts::evaluator::Evaluator;

#[derive(Clone)]
pub struct NeuralNetworkEvaluator<
    G: Game,
    SE: StateEncoder<G>,
    AE: ActionEncoder<G>,
    NN: NeuralNetwork,
> {
    state_encoder: SE,
    action_encoder: AE,
    neural_network: NN,

    _phantom: PhantomData<G>,
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, NN: NeuralNetwork>
    NeuralNetworkEvaluator<G, SE, AE, NN>
{
    pub fn new(state_encoder: SE, action_encoder: AE, neural_network: NN) -> Self {
        NeuralNetworkEvaluator {
            state_encoder,
            action_encoder,
            neural_network,

            _phantom: PhantomData,
        }
    }
}

impl<G, SE, AE, NN> Evaluator<G> for NeuralNetworkEvaluator<G, SE, AE, NN>
where
    G: Game,
    SE: StateEncoder<G>,
    AE: ActionEncoder<G>,
    NN: NeuralNetwork,
{
    fn with_seed(self, _seed: u64) -> Self {
        self
    }

    fn evaluate(&mut self, game: &G) -> Evaluation<G> {
        let state = self.state_encoder.encode(game);

        let Prediction {
            policy_logits,
            value,
        } = self.neural_network.predict(&state);

        let actions = game.get_possible_actions();

        let mut policy = Vec::with_capacity(actions.len());
        let mut total = 0.0;

        for action in actions {
            let action_id = self.action_encoder.encode(&action);

            let value = policy_logits[action_id].exp();
            total += value;

            policy.push(PolicyItem {
                action,
                prior: value,
            });
        }

        for PolicyItem { prior: value, .. } in &mut policy {
            *value /= total.max(f32::EPSILON);
        }

        Evaluation { policy, value }
    }
}
