use crate::game::tic_tac_toe::tic_tac_toe::TicTacToe;
use crate::neural_network::StateEncoder;

#[derive(Clone, Copy, Default)]
pub struct TicTacToeStateEncoder;

impl TicTacToeStateEncoder {
    const PLANE_COUNT: usize = 2;

    pub fn new() -> Self {
        TicTacToeStateEncoder
    }

    fn plane_slice(planes: &mut [f32], plane_index: usize) -> &mut [f32] {
        let plane_size = TicTacToe::BOARD_SIZE * TicTacToe::BOARD_SIZE;

        let start = plane_index * plane_size;
        let end = start + plane_size;

        &mut planes[start..end]
    }

    fn bitboard_to_plane(bits: u16, plane: &mut [f32]) {
        for (i, value) in plane.iter_mut().enumerate() {
            *value = f32::from((bits >> i) & 1);
        }
    }
}

impl StateEncoder<TicTacToe> for TicTacToeStateEncoder {
    fn shape(&self) -> Vec<usize> {
        vec![
            1,
            Self::PLANE_COUNT,
            TicTacToe::BOARD_SIZE,
            TicTacToe::BOARD_SIZE,
        ]
    }

    fn encode(&self, state: &TicTacToe) -> Vec<f32> {
        let plane_size = TicTacToe::BOARD_SIZE * TicTacToe::BOARD_SIZE;
        let plane_count = Self::PLANE_COUNT;

        let mut planes = vec![0.0; plane_size * plane_count];

        Self::bitboard_to_plane(state.player_marks, Self::plane_slice(&mut planes, 0));
        Self::bitboard_to_plane(state.opponent_marks, Self::plane_slice(&mut planes, 1));

        planes
    }
}
