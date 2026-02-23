use wasm_bindgen::prelude::*;

use hermes_engine::Turn;

#[wasm_bindgen(js_name = "Turn")]
#[repr(u8)]
pub enum WasmTurn {
    Player1,
    Player2,
}

impl From<Turn> for WasmTurn {
    fn from(turn: Turn) -> Self {
        match turn {
            Turn::Player1 => WasmTurn::Player1,
            Turn::Player2 => WasmTurn::Player2,
        }
    }
}
