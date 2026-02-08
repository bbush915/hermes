use crate::game::boop::boop::{Boop, Phase};
use crate::neural_network::StateEncoder;

#[derive(Clone, Copy, Default)]
pub struct BoopStateEncoder;

impl BoopStateEncoder {
    const PLANE_COUNT: usize = 10;

    pub fn new() -> Self {
        BoopStateEncoder
    }

    fn plane_slice(planes: &mut [f32], plane_index: usize) -> &mut [f32] {
        let plane_size = Boop::BOARD_SIZE * Boop::BOARD_SIZE;

        let start = plane_index * plane_size;
        let end = start + plane_size;

        &mut planes[start..end]
    }

    fn bitboard_to_plane(bits: u64, plane: &mut [f32]) {
        for (i, value) in plane.iter_mut().enumerate() {
            *value = ((bits >> i) & 1) as f32;
        }
    }

    fn scalar_to_plane(value: f32, plane: &mut [f32]) {
        for entry in plane.iter_mut() {
            *entry = value;
        }
    }
}

impl StateEncoder<Boop> for BoopStateEncoder {
    fn shape(&self) -> Vec<usize> {
        vec![1, Self::PLANE_COUNT, Boop::BOARD_SIZE, Boop::BOARD_SIZE]
    }

    fn encode(&self, state: &Boop) -> Vec<f32> {
        let plane_size = Boop::BOARD_SIZE * Boop::BOARD_SIZE;
        let plane_count = Self::PLANE_COUNT;

        let mut planes = vec![0.0; plane_size * plane_count];

        Self::bitboard_to_plane(state.player_cats, Self::plane_slice(&mut planes, 0));
        Self::bitboard_to_plane(state.player_kittens, Self::plane_slice(&mut planes, 1));
        Self::bitboard_to_plane(state.opponent_cats, Self::plane_slice(&mut planes, 2));
        Self::bitboard_to_plane(state.opponent_kittens, Self::plane_slice(&mut planes, 3));

        Self::scalar_to_plane(
            f32::from(matches!(state.phase, Phase::Place)),
            Self::plane_slice(&mut planes, 4),
        );

        Self::scalar_to_plane(
            f32::from(matches!(state.phase, Phase::Graduate)),
            Self::plane_slice(&mut planes, 5),
        );

        let player_pool = state.player_pool();

        Self::scalar_to_plane(
            f32::from(player_pool.kittens_available) / f32::from(Boop::POOL_SIZE),
            Self::plane_slice(&mut planes, 6),
        );

        Self::scalar_to_plane(
            f32::from(player_pool.cats_available) / f32::from(Boop::POOL_SIZE),
            Self::plane_slice(&mut planes, 7),
        );

        let opponent_pool = state.opponent_pool();

        Self::scalar_to_plane(
            f32::from(opponent_pool.kittens_available) / f32::from(Boop::POOL_SIZE),
            Self::plane_slice(&mut planes, 8),
        );

        Self::scalar_to_plane(
            f32::from(opponent_pool.cats_available) / f32::from(Boop::POOL_SIZE),
            Self::plane_slice(&mut planes, 9),
        );

        planes
    }
}
