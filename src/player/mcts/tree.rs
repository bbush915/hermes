use crate::core::{Game, Turn};

pub struct Tree<G: Game> {
    pub nodes: Vec<Node<G>>,
    pub root_index: usize,

    pub game: G,
}

impl<G: Game> Tree<G> {
    pub fn new(game: G) -> Self {
        let node = Node {
            action: None,
            turn: Turn::PlayerOne,

            parent_index: None,
            child_indices: vec![],

            unexplored_actions: game.get_possible_actions(),

            visits: 0,
            total_value: 0.0,
            prior: 0.0,
        };

        Self {
            nodes: vec![node],
            root_index: 0,

            game,
        }
    }
}

pub struct Node<G: Game> {
    pub action: Option<G::Action>,
    pub turn: Turn,

    pub parent_index: Option<usize>,
    pub child_indices: Vec<usize>,

    pub visits: u32,
    pub total_value: f32,
    pub prior: f32,

    pub unexplored_actions: Vec<G::Action>,
}
