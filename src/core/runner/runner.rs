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
        for game_number in 0..self.games {
            let mut game = G::new();

            let mut turn_number = 0;
            let mut turn = Turn::Player;

            self.sink.emit(RunnerEvent::GameStarted { game_number });

            self.sink.emit(RunnerEvent::TurnStarted {
                game_number,
                turn_number,
                turn,
            });

            loop {
                let choice = match turn {
                    Turn::Player => self.player.choose_action(&game),
                    Turn::Opponent => self.opponent.choose_action(&game),
                };

                if let Some(evaluation) = choice.evaluation {
                    self.sink.emit(RunnerEvent::PositionEvaluated {
                        game_number,
                        turn_number,
                        turn,
                        state: game.clone(),
                        evaluation,
                    });
                }

                let turn_complete = game.apply_action(choice.action);

                self.sink.emit(RunnerEvent::ActionApplied {
                    game_number,
                    turn_number,
                    turn,
                    state: game.clone(),
                    action: choice.action,
                });

                match game.outcome() {
                    Outcome::InProgress => {}
                    outcome => {
                        self.sink.emit(RunnerEvent::GameFinished {
                            game_number,
                            turn_number,
                            turn,
                            outcome,
                        });
                        break;
                    }
                }

                if turn_complete {
                    self.sink.emit(RunnerEvent::TurnFinished {
                        game_number,
                        turn_number,
                        turn,
                    });

                    turn_number += 1;

                    game.end_turn();

                    turn = turn.flip();

                    self.sink.emit(RunnerEvent::TurnStarted {
                        game_number,
                        turn_number,
                        turn,
                    });
                }
            }
        }
    }
}

pub enum RunnerEvent<G: Game> {
    GameStarted {
        game_number: u64,
    },
    TurnStarted {
        game_number: u64,
        turn_number: u64,
        turn: Turn,
    },
    PositionEvaluated {
        game_number: u64,
        turn_number: u64,
        turn: Turn,
        state: G,
        evaluation: Evaluation<G>,
    },
    ActionApplied {
        game_number: u64,
        turn_number: u64,
        turn: Turn,
        state: G,
        action: G::Action,
    },
    TurnFinished {
        game_number: u64,
        turn_number: u64,
        turn: Turn,
    },
    GameFinished {
        game_number: u64,
        turn_number: u64,
        turn: Turn,
        outcome: Outcome,
    },
}
