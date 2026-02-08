use crate::game::tic_tac_toe::TicTacToe;
use crate::game::tic_tac_toe::action::Action;
use crate::neural_network::ActionEncoder;

#[derive(Clone, Copy, Default)]
pub struct TicTacToeActionEncoder;

impl ActionEncoder<TicTacToe> for TicTacToeActionEncoder {
    const ACTION_COUNT: usize = 9;

    fn encode(&self, action: &Action) -> usize {
        match *action {
            Action::Place { index } => index as usize,
        }
    }

    fn decode(&self, action_id: usize) -> Action {
        debug_assert!(action_id < Self::ACTION_COUNT);

        Action::Place {
            index: u8::try_from(action_id).unwrap(),
        }
    }
}
