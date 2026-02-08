use std::marker::PhantomData;

use rand::SeedableRng;
use rand::distributions::WeightedIndex;
use rand::rngs::StdRng;
use rand_distr::{Dirichlet, Distribution};

use crate::core::{Evaluation, Game, PolicyItem};
use crate::player::mcts::evaluator::Evaluator;
use crate::player::mcts::expander::Expander;
use crate::player::mcts::noise::DirichletNoise;
use crate::player::mcts::scorer::Scorer;
use crate::player::mcts::temperature::TemperatureSchedule;
use crate::player::mcts::tree::{Node, Tree};

#[derive(Clone)]
pub struct Mcts<G: Game, E: Evaluator<G>, S: Scorer<G>, X: Expander<G>> {
    rng: StdRng,

    simulations: u32,

    evaluator: E,
    scorer: S,
    expander: X,

    dirichlet_noise: Option<DirichletNoise>,
    temperature_schedule: Option<TemperatureSchedule>,

    _phantom: PhantomData<G>,
}

impl<G: Game, E: Evaluator<G>, S: Scorer<G>, X: Expander<G>> Mcts<G, E, S, X> {
    pub fn new(options: MtcsOptions<G, E, S, X>) -> Self {
        Self {
            rng: StdRng::from_entropy(),

            simulations: options.simulations,

            evaluator: options.evaluator,
            scorer: options.scorer,
            expander: options.expander,

            dirichlet_noise: options.dirichlet_noise,
            temperature_schedule: options.temperature_schedule,

            _phantom: PhantomData,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = StdRng::seed_from_u64(seed);

        self.evaluator = self.evaluator.with_seed(seed);
        self.expander = self.expander.with_seed(seed);

        self
    }

    pub fn with_dirichlet_noise(mut self, dirichlet_noise: DirichletNoise) -> Self {
        self.dirichlet_noise = Some(dirichlet_noise);

        self
    }

    pub fn with_temperature_schedule(mut self, temperature_schedule: TemperatureSchedule) -> Self {
        self.temperature_schedule = Some(temperature_schedule);

        self
    }

    pub fn search(&mut self, game: &G, turn_number: u32) -> SearchResult<G> {
        let mut tree = Tree::new(game.clone());

        for _ in 0..self.simulations {
            let checkpoint = tree.game.create_checkpoint();

            let node_index = self.select(&mut tree);
            let value = self.expand(&mut tree, node_index);
            Self::backpropagate(&mut tree, node_index, value);

            tree.game.restore_checkpoint(checkpoint);
        }

        let evaluation = Self::evaluate(&tree);

        let temperature = self
            .temperature_schedule
            .as_ref()
            .map_or(1.0, |schedule| schedule.get_temperature(turn_number));

        let action = self.choose_action(&evaluation, temperature);

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
                .max_by(|(_, x), (_, y)| x.total_cmp(y))
                .expect("unable to determine best child");

            node_index = child_index;

            if let Some(action) = tree.nodes[node_index].action {
                tree.game.apply_action(action);
            }
        }

        node_index
    }

    fn expand(&mut self, tree: &mut Tree<G>, node_index: usize) -> f32 {
        let node = &tree.nodes[node_index];
        let turn = node.turn;

        let mut evaluation = self.evaluator.evaluate(&tree.game);

        if node_index == tree.root_index {
            self.apply_dirichlet_noise(&mut evaluation);
        }

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
                turn: if turn_complete { turn.advance() } else { turn },

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

    fn apply_dirichlet_noise(&mut self, evaluation: &mut Evaluation<G>) {
        let Some(DirichletNoise { alpha, epsilon }) = self.dirichlet_noise else {
            return;
        };

        if evaluation.policy.len() < 2 {
            return;
        }

        let distribution = Dirichlet::new(vec![alpha; evaluation.policy.len()].as_slice())
            .expect("unable to create dirichlet distribution");

        evaluation
            .policy
            .iter_mut()
            .zip(distribution.sample(&mut self.rng))
            .for_each(|(policy_item, value)| {
                policy_item.prior = (1.0 - epsilon) * policy_item.prior + epsilon * value;
            });
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

    fn evaluate(tree: &Tree<G>) -> Evaluation<G> {
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

    fn choose_action(&mut self, evaluation: &Evaluation<G>, temperature: f32) -> G::Action {
        if temperature == 0.0 {
            return evaluation
                .policy
                .iter()
                .max_by(|x, y| x.prior.total_cmp(&y.prior))
                .expect("unable to choose action")
                .action;
        }

        let weights: Vec<f32> = evaluation
            .policy
            .iter()
            .map(|policy_item| policy_item.prior.powf(1.0 / temperature))
            .collect();

        let distribution =
            WeightedIndex::new(&weights).expect("unable to created weighted distribution");

        let index = distribution.sample(&mut self.rng);

        evaluation.policy[index].action
    }
}

pub struct MtcsOptions<G: Game, E: Evaluator<G>, S: Scorer<G>, X: Expander<G>> {
    pub simulations: u32,

    pub evaluator: E,
    pub scorer: S,
    pub expander: X,

    pub dirichlet_noise: Option<DirichletNoise>,
    pub temperature_schedule: Option<TemperatureSchedule>,

    pub phantom: PhantomData<G>,
}

impl<G: Game, E: Evaluator<G>, S: Scorer<G>, X: Expander<G>> MtcsOptions<G, E, S, X> {
    pub fn new(simulations: u32, evaluator: E, scorer: S, expander: X) -> Self {
        Self {
            simulations,

            evaluator,
            scorer,
            expander,

            dirichlet_noise: None,
            temperature_schedule: None,

            phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn with_dirichlet_noise(mut self, dirichlet_noise: DirichletNoise) -> Self {
        self.dirichlet_noise = Some(dirichlet_noise);

        self
    }

    #[allow(dead_code)]
    pub fn with_temperature_schedule(mut self, schedule: TemperatureSchedule) -> Self {
        self.temperature_schedule = Some(schedule);

        self
    }
}

pub struct SearchResult<G: Game> {
    pub evaluation: Evaluation<G>,
    pub action: G::Action,
}
