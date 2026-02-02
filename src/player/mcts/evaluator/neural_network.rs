use crate::core::{Evaluation, Game, PolicyItem};
use crate::neural_network::{ActionEncoder, NeuralNetwork, StateEncoder};
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

    _marker: std::marker::PhantomData<G>,
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, NN: NeuralNetwork>
    NeuralNetworkEvaluator<G, SE, AE, NN>
{
    pub fn new(state_encoder: SE, action_encoder: AE, neural_network: NN) -> Self {
        NeuralNetworkEvaluator {
            state_encoder,
            action_encoder,
            neural_network,

            _marker: std::marker::PhantomData,
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
    fn evaluate(&mut self, game: &G) -> Evaluation<G> {
        let state = self.state_encoder.encode(game);

        let (policy_logits, value) = self.neural_network.forward(&state);

        let actions = game.get_possible_actions();

        let mut policy = Vec::with_capacity(actions.len());
        let mut sum = 0.0;

        for action in actions {
            let action_id = self.action_encoder.encode(&action);

            let prior = policy_logits[action_id].exp();
            sum += prior;

            policy.push(PolicyItem { action, prior });
        }

        for PolicyItem { prior, .. } in &mut policy {
            *prior /= sum.max(1e-8);
        }

        Evaluation { policy, value }
    }
}
