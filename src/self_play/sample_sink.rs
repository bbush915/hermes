use crate::core::{EventSink, Game, Outcome, PolicyItem, RunnerEvent};
use crate::neural_network::ActionEncoder;
use crate::neural_network::StateEncoder;
use crate::self_play::Sample;

pub struct SampleSink<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, S: EventSink<Sample>> {
    state_encoder: SE,
    action_encoder: AE,

    pending_samples: Vec<PendingSample>,

    sink: S,

    _marker: std::marker::PhantomData<G>,
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, S: EventSink<Sample>>
    SampleSink<G, SE, AE, S>
{
    pub fn new(state_encoder: SE, action_encoder: AE, sink: S) -> Self {
        SampleSink {
            state_encoder,
            action_encoder,

            pending_samples: vec![],

            sink,

            _marker: std::marker::PhantomData,
        }
    }
}

impl<G: Game, SE: StateEncoder<G>, AE: ActionEncoder<G>, S: EventSink<Sample>>
    EventSink<RunnerEvent<G>> for SampleSink<G, SE, AE, S>
{
    fn emit(&mut self, event: RunnerEvent<G>) {
        match event {
            RunnerEvent::GameStarted { .. } => {
                self.pending_samples.clear();
            }
            RunnerEvent::PositionEvaluated {
                state, evaluation, ..
            } => {
                let state = self.state_encoder.encode(&state);

                let mut policy = vec![0.0; self.action_encoder.size()];

                for PolicyItem { action, prior } in evaluation.policy {
                    let action_index = self.action_encoder.encode(&action);
                    policy[action_index] = prior;
                }

                self.pending_samples.push(PendingSample { state, policy });
            }
            RunnerEvent::GameFinished { outcome, .. } => {
                let value = match outcome {
                    Outcome::Win => 1.0,
                    Outcome::Loss => -1.0,
                    Outcome::Draw => 0.0,
                    Outcome::InProgress => unreachable!(),
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
