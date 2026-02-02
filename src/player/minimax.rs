use crate::core::{Choice, Game, Outcome, Player};

#[derive(Clone)]
pub struct MinimaxPlayer {
    depth: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Objective {
    Maximize,
    Minimize,
}

impl Objective {
    fn flip(self) -> Self {
        match self {
            Objective::Maximize => Objective::Minimize,
            Objective::Minimize => Objective::Maximize,
        }
    }

    fn sign(self) -> f32 {
        match self {
            Objective::Maximize => 1.0,
            Objective::Minimize => -1.0,
        }
    }
}

impl MinimaxPlayer {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }

    fn minimax<G: Game>(
        &self,
        game: &mut G,
        depth: usize,
        objective: Objective,
        alpha: f32,
        beta: f32,
    ) -> (f32, Option<G::Action>) {
        let outcome = game.outcome();

        if depth == 0 || outcome != Outcome::InProgress {
            let value = match outcome {
                Outcome::Win => objective.sign(),
                Outcome::Loss => -objective.sign(),
                _ => 0.0,
            };

            return (value, None);
        }

        let mut best_value = match objective {
            Objective::Maximize => f32::NEG_INFINITY,
            Objective::Minimize => f32::INFINITY,
        };

        let mut best_action = None;

        let mut alpha = alpha;
        let mut beta = beta;

        let checkpoint = game.create_checkpoint();

        for action in game.get_possible_actions() {
            let turn_ended = game.apply_action(action);

            let (value, _) = self.minimax(
                game,
                depth - 1,
                if turn_ended {
                    objective.flip()
                } else {
                    objective
                },
                alpha,
                beta,
            );

            game.restore_checkpoint(checkpoint);

            match objective {
                Objective::Maximize => {
                    if value > best_value {
                        best_value = value;
                        best_action = Some(action);
                    }

                    alpha = alpha.max(best_value);
                }
                Objective::Minimize => {
                    if value < best_value {
                        best_value = value;
                        best_action = Some(action);
                    }

                    beta = beta.min(best_value);
                }
            }

            if beta <= alpha {
                break;
            }
        }

        (best_value, best_action)
    }
}

impl<G: Game> Player<G> for MinimaxPlayer {
    fn name(&self) -> &str {
        "Minimax with Alpha-Beta Pruning"
    }

    fn choose_action(&mut self, game: &G) -> Choice<G> {
        let (_, action) = self.minimax(
            &mut game.clone(),
            self.depth,
            Objective::Maximize,
            f32::NEG_INFINITY,
            f32::INFINITY,
        );

        let action = action.expect("no legal actions available");

        Choice {
            evaluation: None,
            action,
        }
    }
}
