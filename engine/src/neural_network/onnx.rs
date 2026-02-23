use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;

use tract_onnx::prelude::*;

use crate::core::Game;
use crate::neural_network::neural_network::{NeuralNetwork, Prediction};
use crate::neural_network::state_encoder::StateEncoder;

type TractModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

#[derive(Clone)]
pub struct OnnxNeuralNetwork<G: Game, SE: StateEncoder<G>> {
    model: Arc<TractModel>,

    state_encoder: SE,

    _phantom: PhantomData<G>,
}

impl<G: Game, SE: StateEncoder<G>> OnnxNeuralNetwork<G, SE> {
    pub fn new(path: impl AsRef<std::path::Path>, state_encoder: SE) -> Result<Self, Box<dyn Error>> {
        let model = tract_onnx::onnx()
            .model_for_path(path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
            state_encoder,
            _phantom: PhantomData,
        })
    }

    pub fn new_from_bytes(bytes: &[u8], state_encoder: SE) -> Result<Self, Box<dyn Error>> {
        let model = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(bytes))?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
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
        let shape = self.state_encoder.shape();

        let tensor: Tensor =
            tract_ndarray::Array::from_shape_vec(tract_ndarray::IxDyn(&shape), input.to_vec())
                .expect("failed to create input tensor")
                .into();

        let result = self.model.run(tvec!(tensor.into())).expect("failed to run model");

        let policy_logits: Vec<f32> = result[0]
            .to_array_view::<f32>()
            .expect("failed to extract policy")
            .iter()
            .copied()
            .collect();

        let value = *result[1]
            .to_array_view::<f32>()
            .expect("failed to extract value")
            .iter()
            .next()
            .expect("value output is empty");

        Prediction {
            policy_logits,
            value,
        }
    }
}