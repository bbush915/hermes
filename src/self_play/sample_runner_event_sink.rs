use std::marker::PhantomData;

use crate::core::{
    EventSink, Game, Outcome, PolicyItem, RunnerEvent, RunnerEventContext, RunnerEventKind, Turn,
};
use crate::neural_network::{ActionEncoder, StateEncoder};
use crate::self_play::Sample;

pub struct SampleRunnerEventSink<
    G: Game,
    SE: StateEncoder<G>,
    AE: ActionEncoder<G>,
    S: EventSink<Sample>,
> {
    state_encoder: SE,
    action_encoder: AE,

    use_symmetries: bool,
    pending_samples: Vec<PendingSample>,

    sink: S,

    _phantom: PhantomData<G>,
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, S: EventSink<Sample>>
    SampleRunnerEventSink<G, SE, AE, S>
{
    pub fn new(state_encoder: SE, action_encoder: AE, use_symmetries: bool, sink: S) -> Self {
        SampleRunnerEventSink {
            state_encoder,
            action_encoder,

            use_symmetries,
            pending_samples: vec![],

            sink,

            _phantom: PhantomData,
        }
    }
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, S: EventSink<Sample>>
    EventSink<RunnerEvent<G>> for SampleRunnerEventSink<G, SE, AE, S>
{
    fn emit(&mut self, event: RunnerEvent<G>) {
        let RunnerEvent { kind, context } = event;

        let Some(RunnerEventContext { game, turn, .. }) = context else {
            return;
        };

        match kind {
            RunnerEventKind::GameStarted => {
                self.pending_samples.clear();
            }
            RunnerEventKind::PositionEvaluated { evaluation } => {
                let symmetries = if self.use_symmetries {
                    game.symmetries()
                } else {
                    1
                };

                for symmetry in 0..symmetries {
                    let state = self.state_encoder.encode(&game.transform(symmetry));

                    let mut policy = vec![0.0; self.action_encoder.size()];

                    for PolicyItem { action, prior } in &evaluation.policy {
                        let action_index = self
                            .action_encoder
                            .encode(&game.transform_action(*action, symmetry));

                        policy[action_index] = *prior;
                    }

                    self.pending_samples.push(PendingSample { state, policy });
                }
            }
            RunnerEventKind::GameFinished { outcome } => {
                let value = match (outcome, turn) {
                    (Outcome::InProgress, _) => unreachable!(),
                    (Outcome::Win, Turn::PlayerOne) | (Outcome::Loss, Turn::PlayerTwo) => 1.0,
                    (Outcome::Win, Turn::PlayerTwo) | (Outcome::Loss, Turn::PlayerOne) => -1.0,
                    (Outcome::Draw, _) => 0.0,
                };

                for PendingSample { state, policy } in self.pending_samples.drain(..) {
                    let sample = Sample {
                        state,
                        policy,
                        value,
                    };

                    self.sink.emit(sample);
                }
            }
            _ => {}
        }
    }
}

struct PendingSample {
    pub state: Vec<f32>,
    pub policy: Vec<f32>,
}
