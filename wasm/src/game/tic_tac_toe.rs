use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use hermes_engine::tic_tac_toe::{TicTacToe, TicTacToeAction, TicTacToePhase};
use hermes_engine::{Game, ManualPlayer, MinimaxPlayer, Outcome, Player, RandomPlayer, Turn};

use crate::game::outcome::WasmOutcome;
use crate::game::turn::WasmTurn;
use crate::player::PlayerKind;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WasmTicTacToeAction {
    Place { index: u8 },
}

impl From<TicTacToeAction> for WasmTicTacToeAction {
    fn from(action: TicTacToeAction) -> Self {
        match action {
            TicTacToeAction::Place { index } => WasmTicTacToeAction::Place { index },
        }
    }
}

impl From<WasmTicTacToeAction> for TicTacToeAction {
    fn from(action: WasmTicTacToeAction) -> Self {
        match action {
            WasmTicTacToeAction::Place { index } => TicTacToeAction::Place { index },
        }
    }
}

#[wasm_bindgen(js_name = "TicTacToePhase")]
#[repr(u8)]
pub enum WasmTicTacToePhase {
    Place,
}

impl From<TicTacToePhase> for WasmTicTacToePhase {
    fn from(phase: TicTacToePhase) -> Self {
        match phase {
            TicTacToePhase::Place => WasmTicTacToePhase::Place,
        }
    }
}

#[wasm_bindgen(js_name = "TicTacToeSquare")]
#[repr(u8)]
pub enum WasmTicTacToeSquare {
    Empty,
    Player1,
    Player2,
}

#[wasm_bindgen(js_name = "TicTacToe")]
pub struct WasmTicTacToe {
    game: TicTacToe,

    turn: Turn,
    turn_number: u32,

    player_1_kind: PlayerKind,
    player_1_impl: Box<dyn Player<TicTacToe>>,
    player_2_kind: PlayerKind,
    player_2_impl: Box<dyn Player<TicTacToe>>,

    queued_action: Option<TicTacToeAction>,
}

#[wasm_bindgen(js_class = "TicTacToe")]
impl WasmTicTacToe {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTicTacToe {
        WasmTicTacToe {
            game: TicTacToe::new(),

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
        self.player_1_impl = WasmTicTacToe::make_player(kind);
    }

    #[wasm_bindgen(getter)]
    pub fn player_2_kind(&self) -> PlayerKind {
        self.player_2_kind
    }

    #[wasm_bindgen(setter)]
    pub fn set_player_2(&mut self, kind: PlayerKind) {
        self.player_2_kind = kind;
        self.player_2_impl = WasmTicTacToe::make_player(kind);
    }

    #[wasm_bindgen(getter)]
    pub fn turn(&self) -> WasmTurn {
        self.turn.into()
    }

    #[wasm_bindgen(getter)]
    pub fn phase(&self) -> WasmTicTacToePhase {
        self.game.phase.into()
    }

    #[wasm_bindgen(getter)]
    pub fn outcome(&self) -> WasmOutcome {
        self.game.outcome().into()
    }

    #[wasm_bindgen(getter)]
    pub fn board(&self) -> Vec<WasmTicTacToeSquare> {
        let (player_1_marks, player_2_marks) = match self.turn {
            Turn::Player1 => (self.game.player_marks, self.game.opponent_marks),
            Turn::Player2 => (self.game.opponent_marks, self.game.player_marks),
        };

        (0..TicTacToe::BOARD_SIZE * TicTacToe::BOARD_SIZE)
            .map(|i| {
                let mask = 1u16 << i;

                if player_1_marks & mask != 0 {
                    WasmTicTacToeSquare::Player1
                } else if player_2_marks & mask != 0 {
                    WasmTicTacToeSquare::Player2
                } else {
                    WasmTicTacToeSquare::Empty
                }
            })
            .collect()
    }

    #[wasm_bindgen]
    pub fn get_possible_actions(&self) -> JsValue {
        let actions: Vec<WasmTicTacToeAction> = self
            .game
            .get_possible_actions()
            .into_iter()
            .map(WasmTicTacToeAction::from)
            .collect();

        serde_wasm_bindgen::to_value(&actions).expect("failed to serialize actions")
    }

    #[wasm_bindgen]
    pub fn queue_action(&mut self, action: JsValue) {
        let wasm_action: WasmTicTacToeAction =
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

    fn make_player(kind: PlayerKind) -> Box<dyn Player<TicTacToe>> {
        match kind {
            PlayerKind::Manual => Box::new(ManualPlayer::new()),
            PlayerKind::Random => Box::new(RandomPlayer::new()),
            PlayerKind::Minimax => Box::new(MinimaxPlayer::new(10)),
        }
    }
}
