use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PlayerKind {
    Manual,
    Random,
    Minimax,
}
