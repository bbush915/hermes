use crate::core::evaluation::Evaluation;
use crate::core::event::EventSink;
use crate::core::game::{Game, Outcome};
use crate::core::player::Player;
use crate::core::turn::Turn;

pub struct Runner<G, P, O, S>
where
    G: Game,
    P: Player<G>,
    O: Player<G>,
    S: EventSink<RunnerEvent<G>>,
{
    games: u64,

    player: P,
    opponent: O,
    sink: S,

    _marker: std::marker::PhantomData<G>,
}

impl<G, P, O, S> Runner<G, P, O, S>
where
    G: Game,
    P: Player<G>,
    O: Player<G>,
    S: EventSink<RunnerEvent<G>>,
{
    pub fn new(games: u64, player: P, opponent: O, sink: S) -> Self {
        Self {
            games,

            player,
            opponent,
            sink,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn run(&mut self) {
        for game_id in 0..self.games {
            let mut game = G::new();
            let mut turn = Turn::Player;

            self.sink.emit(RunnerEvent::GameStarted { game_id });

            loop {
                let choice = match turn {
                    Turn::Player => self.player.choose_action(&game),
                    Turn::Opponent => self.opponent.choose_action(&game),
                };

                if let Some(evaluation) = choice.evaluation {
                    self.sink.emit(RunnerEvent::PositionEvaluated {
                        state: game.clone(),
                        evaluation,
                    });
                }

                let turn_ended = game.apply_action(choice.action);

                if turn_ended {
                    turn = turn.flip();
                }

                self.sink.emit(RunnerEvent::ActionApplied {
                    state: game.clone(),
                    turn,
                    action: choice.action,
                });

                match game.outcome() {
                    Outcome::InProgress => {}
                    outcome => {
                        self.sink.emit(RunnerEvent::GameFinished {
                            game_id,
                            turn,
                            outcome,
                        });
                        break;
                    }
                }
            }
        }
    }
}

pub enum RunnerEvent<G: Game> {
    GameStarted {
        game_id: u64,
    },
    PositionEvaluated {
        state: G,
        evaluation: Evaluation<G>,
    },
    ActionApplied {
        state: G,
        turn: Turn,
        action: G::Action,
    },
    GameFinished {
        game_id: u64,
        turn: Turn,
        outcome: Outcome,
    },
}
