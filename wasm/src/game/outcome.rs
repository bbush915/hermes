use wasm_bindgen::prelude::*;

use hermes_engine::Outcome;

#[wasm_bindgen(js_name = "Outcome")]
#[repr(u8)]
pub enum WasmOutcome {
    InProgress,
    Win,
    Loss,
    Draw,
}

impl From<Outcome> for WasmOutcome {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::InProgress => WasmOutcome::InProgress,
            Outcome::Win => WasmOutcome::Win,
            Outcome::Loss => WasmOutcome::Loss,
            Outcome::Draw => WasmOutcome::Draw,
        }
    }
}
