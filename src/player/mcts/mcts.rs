use std::marker::PhantomData;
use std::vec;

use crate::core::{Evaluation, Game, PolicyItem, Turn};
use crate::player::mcts::evaluator::Evaluator;
use crate::player::mcts::expander::Expander;
use crate::player::mcts::scorer::Scorer;

#[derive(Clone)]
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

    pub fn search(&mut self, game: &G) -> SearchResult<G> {
        let mut tree = Tree::new(game.clone());

        for _ in 0..self.simulations {
            let checkpoint = tree.game.create_checkpoint();

            let node_index = self.select(&mut tree);
            let value = self.expand_and_evaluate(&mut tree, node_index);
            Self::backpropagate(&mut tree, node_index, value);

            tree.game.restore_checkpoint(checkpoint);
        }

        let evaluation = Self::make_evaluation(&tree);

        let action = evaluation
            .policy
            .iter()
            .max_by(|x, y| x.prior.partial_cmp(&y.prior).unwrap())
            .unwrap()
            .action;

        SearchResult { evaluation, action }
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

        for PolicyItem { action, prior } in expansion {
            let checkpoint = tree.game.create_checkpoint();

            let turn_complete = tree.game.apply_action(action);

            if turn_complete {
                tree.game.end_turn();
            }

            let child_node = Node {
                action: Some(action),
                turn: if turn_complete { turn.flip() } else { turn },

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

    fn backpropagate(tree: &mut Tree<G>, mut node_index: usize, value: f32) {
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

    fn make_evaluation(tree: &Tree<G>) -> Evaluation<G> {
        let root = &tree.nodes[tree.root_index];

        let total_visits: u32 = root
            .child_indices
            .iter()
            .map(|&i| tree.nodes[i].visits)
            .sum();

        let policy = root
            .child_indices
            .iter()
            .filter_map(|&i| {
                let node = &tree.nodes[i];

                let action = node.action?;
                let prior = node.visits as f32 / total_visits.max(1) as f32;

                Some(PolicyItem { action, prior })
            })
            .collect();

        let value = root.total_value / root.visits.max(1) as f32;

        Evaluation { policy, value }
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

pub struct SearchResult<G: Game> {
    pub evaluation: Evaluation<G>,
    pub action: G::Action,
}
