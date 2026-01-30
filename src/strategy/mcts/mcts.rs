use std::marker::PhantomData;
use std::vec;

use crate::core::{Game, Turn};
use crate::strategy::mcts::evaluator::{Evaluator, PolicyEntry};
use crate::strategy::mcts::expander::Expander;
use crate::strategy::mcts::scorer::Scorer;

pub struct Mcts<G: Game, E: Evaluator<G>, S: Scorer<G>, X: Expander<G>> {
    simulations: u32,

    evaluator: E,
    scorer: S,
    expander: X,

    _phantom: PhantomData<G>,
}

impl<G: Game, E: Evaluator<G>, S: Scorer<G>, X: Expander<G>> Mcts<G, E, S, X> {
    pub fn new(simulations: u32, evaluator: E, scorer: S, expander: X) -> Self {
        Self {
            simulations,

            evaluator,
            scorer,
            expander,

            _phantom: PhantomData,
        }
    }

    pub fn search(&mut self, game: &G) -> G::Action {
        let mut tree = Tree::new(game.clone());

        for _ in 0..self.simulations {
            let checkpoint = tree.game.create_checkpoint();

            let node_index = self.select(&mut tree);
            let value = self.expand_and_evaluate(&mut tree, node_index);
            self.backpropagate(&mut tree, node_index, value);

            tree.game.restore_checkpoint(checkpoint);
        }

        tree.nodes[tree.root_index]
            .child_indices
            .iter()
            .copied()
            .max_by_key(|&child_index| tree.nodes[child_index].visits)
            .and_then(|child_index| tree.nodes[child_index].action)
            .expect("no legal actions available")
    }

    fn select(&self, tree: &mut Tree<G>) -> usize {
        let mut node_index = tree.root_index;

        loop {
            let node = &tree.nodes[node_index];

            if node.child_indices.is_empty() || !node.unexplored_actions.is_empty() {
                break;
            }

            let (child_index, _) = node
                .child_indices
                .iter()
                .map(|&child_index| {
                    let score = self.scorer.score(node, &tree.nodes[child_index]);

                    (child_index, score)
                })
                .max_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
                .unwrap();

            node_index = child_index;

            if let Some(action) = tree.nodes[node_index].action {
                tree.game.apply_action(action);
            }
        }

        node_index
    }

    fn expand_and_evaluate(&mut self, tree: &mut Tree<G>, node_index: usize) -> f32 {
        let node = &tree.nodes[node_index];
        let turn = node.turn;

        let evaluation = self.evaluator.evaluate(&tree.game);

        let value = if turn == tree.nodes[tree.root_index].turn {
            evaluation.value
        } else {
            -evaluation.value
        };

        let expansion = self
            .expander
            .expand(&mut tree.nodes[node_index], &evaluation);

        for PolicyEntry { action, prior } in expansion {
            let checkpoint = tree.game.create_checkpoint();

            let turn_ended = tree.game.apply_action(action);

            let child_node = Node {
                action: Some(action),
                turn: if turn_ended { turn.flip() } else { turn },

                parent_index: Some(node_index),
                child_indices: vec![],

                unexplored_actions: tree.game.get_possible_actions(),

                visits: 0,
                total_value: 0.0,
                prior,
            };

            tree.game.restore_checkpoint(checkpoint);

            let child_index = tree.nodes.len();

            tree.nodes.push(child_node);
            tree.nodes[node_index].child_indices.push(child_index);
        }

        value
    }

    fn backpropagate(&self, tree: &mut Tree<G>, mut node_index: usize, value: f32) {
        loop {
            let node = &mut tree.nodes[node_index];

            node.visits += 1;
            node.total_value += value;

            if let Some(parent_index) = node.parent_index {
                node_index = parent_index;
            } else {
                break;
            }
        }
    }
}

struct Tree<G: Game> {
    nodes: Vec<Node<G>>,
    root_index: usize,

    game: G,
}

impl<G: Game> Tree<G> {
    pub fn new(game: G) -> Self {
        let node = Node {
            action: None,
            turn: Turn::Player,

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
    action: Option<G::Action>,
    turn: Turn,

    parent_index: Option<usize>,
    child_indices: Vec<usize>,

    pub visits: u32,
    pub total_value: f32,
    pub prior: f32,

    pub unexplored_actions: Vec<G::Action>,
}
