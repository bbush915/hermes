use std::error::Error;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, Mutex};

use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;

use crate::core::Game;
use crate::neural_network::neural_network::{NeuralNetwork, Prediction};
use crate::neural_network::state_encoder::StateEncoder;

#[derive(Clone)]
pub struct OnnxNeuralNetwork<G: Game, SE: StateEncoder<G>> {
    session: Arc<Mutex<Session>>,

    state_encoder: SE,

    _phantom: PhantomData<G>,
}

impl<G: Game, SE: StateEncoder<G>> OnnxNeuralNetwork<G, SE> {
    pub fn new(path: impl AsRef<Path>, state_encoder: SE) -> Result<Self, Box<dyn Error>> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(path)?;

        Ok(Self {
            session: Arc::new(Mutex::new(session)),

            state_encoder,

            _phantom: PhantomData,
        })
    }
}

impl<G: Game, SE: StateEncoder<G>> NeuralNetwork for OnnxNeuralNetwork<G, SE> {
    fn with_seed(self, _seed: u64) -> Self {
        self
    }

    fn predict(&mut self, input: &[f32]) -> Prediction {
        let tensor = Tensor::from_array((self.state_encoder.shape(), input.to_vec()))
            .expect("failed to create tensor");

        let mut binding = self.session.lock().expect("failed to lock session");

        let outputs = binding
            .run(ort::inputs![tensor])
            .expect("failed to run session");

        let policy_logits: Vec<f32> = outputs["policy"]
            .try_extract_array::<f32>()
            .expect("failed to extract policy")
            .iter()
            .copied()
            .collect();

        let value = outputs["value"]
            .try_extract_array::<f32>()
            .expect("failed to extract value")[0];

        Prediction {
            policy_logits,
            value,
        }
    }
}
