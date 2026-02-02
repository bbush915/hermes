use crate::game::tic_tac_toe::tic_tac_toe::TicTacToe;
use crate::neural_network::StateEncoder;

#[derive(Clone)]
pub struct TicTacToeStateEncoder;

impl TicTacToeStateEncoder {
    const PLANE_COUNT: usize = 2;

    pub fn new() -> Self {
        TicTacToeStateEncoder
    }

    #[inline(always)]
    fn plane_slice(planes: &mut [f32], plane_index: usize) -> &mut [f32] {
        let plane_size = TicTacToe::BOARD_SIZE * TicTacToe::BOARD_SIZE;

        let start = plane_index * plane_size;
        let end = start + plane_size;

        &mut planes[start..end]
    }

    #[inline(always)]
    fn bitboard_to_plane(bits: u16, plane: &mut [f32]) {
        for i in 0..TicTacToe::BOARD_SIZE * TicTacToe::BOARD_SIZE {
            plane[i] = ((bits >> i) & 1) as f32;
        }
    }

    #[inline(always)]
    fn scalar_to_plane(value: f32, plane: &mut [f32]) {
        for entry in plane.iter_mut() {
            *entry = value;
        }
    }
}

impl StateEncoder<TicTacToe> for TicTacToeStateEncoder {
    fn size(&self) -> (usize, usize, usize) {
        (
            TicTacToe::BOARD_SIZE,
            TicTacToe::BOARD_SIZE,
            Self::PLANE_COUNT,
        )
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
