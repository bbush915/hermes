use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use hermes_engine::boop::{Boop, BoopAction, BoopPhase, BoopPiece};
use hermes_engine::{Game, ManualPlayer, MinimaxPlayer, Outcome, Player, RandomPlayer, Turn};

use crate::game::outcome::WasmOutcome;
use crate::game::turn::WasmTurn;
use crate::player::PlayerKind;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WasmBoopAction {
    Place { square: u8, is_cat: bool },
    Graduate { squares: Vec<u8> },
}

impl From<BoopAction> for WasmBoopAction {
    fn from(action: BoopAction) -> Self {
        match action {
            BoopAction::Place { piece, index } => WasmBoopAction::Place {
                square: index,
                is_cat: piece == BoopPiece::Cat,
            },
            BoopAction::Graduate { mask } => WasmBoopAction::Graduate {
                squares: (0..36u8).filter(|&i| (mask >> i) & 1 == 1).collect(),
            },
        }
    }
}

impl From<WasmBoopAction> for BoopAction {
    fn from(action: WasmBoopAction) -> Self {
        match action {
            WasmBoopAction::Place { square, is_cat } => BoopAction::Place {
                piece: if is_cat {
                    BoopPiece::Cat
                } else {
                    BoopPiece::Kitten
                },
                index: square,
            },
            WasmBoopAction::Graduate { squares } => BoopAction::Graduate {
                mask: squares
                    .iter()
                    .fold(0u64, |mask, &index| mask | (1u64 << index)),
            },
        }
    }
}

#[wasm_bindgen(js_name = "Boop")]
pub struct WasmBoop {
    game: Boop,

    turn: Turn,
    turn_number: u32,

    player_1_kind: PlayerKind,
    player_1_impl: Box<dyn Player<Boop>>,
    player_2_kind: PlayerKind,
    player_2_impl: Box<dyn Player<Boop>>,

    queued_action: Option<BoopAction>,
}

#[wasm_bindgen(js_name = "BoopPhase")]
#[repr(u8)]
pub enum WasmBoopPhase {
    Place,
    Graduate,
}

impl From<BoopPhase> for WasmBoopPhase {
    fn from(phase: BoopPhase) -> Self {
        match phase {
            BoopPhase::Place => WasmBoopPhase::Place,
            BoopPhase::Graduate => WasmBoopPhase::Graduate,
        }
    }
}

#[wasm_bindgen(js_name = "BoopSquare")]
#[repr(u8)]
pub enum WasmBoopSquare {
    Empty,
    Player1Kitten,
    Player1Cat,
    Player2Kitten,
    Player2Cat,
}

#[wasm_bindgen(js_class = "Boop")]
impl WasmBoop {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmBoop {
        WasmBoop {
            game: Boop::new(),

            turn: Turn::Player1,
            turn_number: 0,

            player_1_kind: PlayerKind::Manual,
            player_1_impl: Box::new(ManualPlayer::new()),
            player_2_kind: PlayerKind::Manual,
            player_2_impl: Box::new(ManualPlayer::new()),

            queued_action: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn player_1_kind(&self) -> PlayerKind {
        self.player_1_kind
    }

    #[wasm_bindgen(setter)]
    pub fn set_player_1(&mut self, kind: PlayerKind) {
        self.player_1_kind = kind;
        self.player_1_impl = WasmBoop::make_player(kind);
    }

    #[wasm_bindgen(getter)]
    pub fn player_2_kind(&self) -> PlayerKind {
        self.player_2_kind
    }

    #[wasm_bindgen(setter)]
    pub fn set_player_2(&mut self, kind: PlayerKind) {
        self.player_2_kind = kind;
        self.player_2_impl = WasmBoop::make_player(kind);
    }

    #[wasm_bindgen(getter)]
    pub fn turn(&self) -> WasmTurn {
        self.turn.into()
    }

    #[wasm_bindgen(getter)]
    pub fn phase(&self) -> WasmBoopPhase {
        self.game.phase.into()
    }

    #[wasm_bindgen(getter)]
    pub fn outcome(&self) -> WasmOutcome {
        self.game.outcome().into()
    }

    #[wasm_bindgen(getter)]
    pub fn board(&self) -> Vec<WasmBoopSquare> {
        let (player_1_cats, player_1_kittens, player_2_cats, player_2_kittens) = match self.turn {
            Turn::Player1 => (
                self.game.player_cats,
                self.game.player_kittens,
                self.game.opponent_cats,
                self.game.opponent_kittens,
            ),
            Turn::Player2 => (
                self.game.opponent_cats,
                self.game.opponent_kittens,
                self.game.player_cats,
                self.game.player_kittens,
            ),
        };

        (0..Boop::BOARD_SIZE * Boop::BOARD_SIZE)
            .map(|i| {
                let mask = 1u64 << i;

                if player_1_cats & mask != 0 {
                    WasmBoopSquare::Player1Cat
                } else if player_1_kittens & mask != 0 {
                    WasmBoopSquare::Player1Kitten
                } else if player_2_cats & mask != 0 {
                    WasmBoopSquare::Player2Cat
                } else if player_2_kittens & mask != 0 {
                    WasmBoopSquare::Player2Kitten
                } else {
                    WasmBoopSquare::Empty
                }
            })
            .collect()
    }

    #[wasm_bindgen]
    pub fn get_possible_actions(&self) -> JsValue {
        let actions: Vec<WasmBoopAction> = self
            .game
            .get_possible_actions()
            .into_iter()
            .map(WasmBoopAction::from)
            .collect();

        serde_wasm_bindgen::to_value(&actions).expect("failed to serialize actions")
    }

    #[wasm_bindgen]
    pub fn queue_action(&mut self, action: JsValue) {
        let wasm_action: WasmBoopAction =
            serde_wasm_bindgen::from_value(action).expect("failed to deserialize action");

        self.queued_action = Some(wasm_action.into());
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> WasmOutcome {
        let current_kind = match self.turn {
            Turn::Player1 => self.player_1_kind,
            Turn::Player2 => self.player_2_kind,
        };

        let action = if let PlayerKind::Manual = current_kind {
            self.queued_action
                .take()
                .expect("no action queued for manual player")
        } else {
            let player = match self.turn {
                Turn::Player1 => &mut self.player_1_impl,
                Turn::Player2 => &mut self.player_2_impl,
            };

            let choice = player.choose_action(&self.game, self.turn_number);

            choice.action
        };

        let turn_complete = self.game.apply_action(action);

        if self.game.outcome() != Outcome::InProgress {
            return self.game.outcome().into();
        }

        if turn_complete {
            self.game.end_turn();

            self.turn = self.turn.advance();
            self.turn_number += 1;
        }

        WasmOutcome::InProgress
    }

    fn make_player(kind: PlayerKind) -> Box<dyn Player<Boop>> {
        match kind {
            PlayerKind::Manual => Box::new(ManualPlayer::new()),
            PlayerKind::Random => Box::new(RandomPlayer::new()),
            PlayerKind::Minimax => Box::new(MinimaxPlayer::new(3)),
        }
    }
}
