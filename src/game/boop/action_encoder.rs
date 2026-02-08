use crate::game::boop::Boop;
use crate::game::boop::action::{Action, Piece};
use crate::neural_network::ActionEncoder;

#[derive(Clone, Copy, Default)]
pub struct BoopActionEncoder;

impl BoopActionEncoder {
    const PLACE_COUNT: usize = 72;
    const GRADUATE_THREE_IN_A_ROW_COUNT: usize = 80;
    const GRADUATE_SINGLE_COUNT: usize = 36;

    pub fn new() -> Self {
        BoopActionEncoder
    }
}

impl ActionEncoder<Boop> for BoopActionEncoder {
    const ACTION_COUNT: usize =
        Self::PLACE_COUNT + Self::GRADUATE_THREE_IN_A_ROW_COUNT + Self::GRADUATE_SINGLE_COUNT;

    fn encode(&self, action: &Action) -> usize {
        match *action {
            Action::Place { piece, index } => {
                let piece_id = match piece {
                    Piece::Kitten => 0,
                    Piece::Cat => 1,
                };
                index as usize * 2 + piece_id
            }

            Action::Graduate { mask } => {
                if let Some(index) = Boop::THREE_IN_A_ROW_MASKS.iter().position(|&x| x == mask) {
                    Self::PLACE_COUNT + index
                } else {
                    let index = mask.trailing_zeros() as usize;

                    Self::PLACE_COUNT + Self::GRADUATE_THREE_IN_A_ROW_COUNT + index
                }
            }
        }
    }

    fn decode(&self, action_id: usize) -> Action {
        debug_assert!(action_id < Self::ACTION_COUNT);

        if action_id < Self::PLACE_COUNT {
            let index = u8::try_from(action_id / 2).unwrap();

            let piece = if action_id.is_multiple_of(2) {
                Piece::Kitten
            } else {
                Piece::Cat
            };

            Action::Place { piece, index }
        } else if action_id - Self::PLACE_COUNT < Self::GRADUATE_THREE_IN_A_ROW_COUNT {
            let index = action_id - Self::PLACE_COUNT;

            Action::Graduate {
                mask: Boop::THREE_IN_A_ROW_MASKS[index],
            }
        } else {
            let index = action_id - Self::PLACE_COUNT - Self::GRADUATE_THREE_IN_A_ROW_COUNT;

            Action::Graduate {
                mask: 1u64 << index,
            }
        }
    }
}
