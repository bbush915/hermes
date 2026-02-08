use std::marker::PhantomData;

use crate::core::Evaluation;
use crate::core::event::EventSink;
use crate::core::game::{Game, Outcome};
use crate::core::player::Player;
use crate::core::turn::Turn;

pub struct Runner<G, P1, P2, S>
where
    G: Game,
    P1: Player<G>,
    P2: Player<G>,
    S: EventSink<RunnerEvent<G>>,
{
    games: u32,
    max_turns: Option<u32>,

    player_1: P1,
    player_2: P2,

    sink: S,

    _phantom: PhantomData<G>,
}

impl<G, P1, P2, S> Runner<G, P1, P2, S>
where
    G: Game,
    P1: Player<G>,
    P2: Player<G>,
    S: EventSink<RunnerEvent<G>>,
{
    pub fn new(games: u32, player_1: P1, player_2: P2, sink: S) -> Self {
        Self {
            games,
            max_turns: None,

            player_1,
            player_2,

            sink,

            _phantom: PhantomData,
        }
    }

    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = Some(max_turns);

        self
    }

    pub fn run(&mut self) {
        self.sink.emit(RunnerEvent {
            kind: RunnerEventKind::RunnerStarted,
            context: None,
        });

        for game_number in 0..self.games {
            let mut game = G::new();

            let mut turn_number = 0;

            let mut turn = if game_number % 2 == 0 {
                Turn::PlayerOne
            } else {
                Turn::PlayerTwo
            };

            self.sink.emit(RunnerEvent {
                kind: RunnerEventKind::GameStarted,
                context: Some(RunnerEventContext {
                    game_number,
                    game: game.clone(),
                    turn_number,
                    turn,
                }),
            });

            self.sink.emit(RunnerEvent {
                kind: RunnerEventKind::TurnStarted,
                context: Some(RunnerEventContext {
                    game_number,
                    game: game.clone(),
                    turn_number,
                    turn,
                }),
            });

            loop {
                let choice = match turn {
                    Turn::PlayerOne => self.player_1.choose_action(&game, turn_number),
                    Turn::PlayerTwo => self.player_2.choose_action(&game, turn_number),
                };

                if let Some(evaluation) = choice.evaluation {
                    self.sink.emit(RunnerEvent {
                        kind: RunnerEventKind::PositionEvaluated { evaluation },
                        context: Some(RunnerEventContext {
                            game_number,
                            game: game.clone(),
                            turn_number,
                            turn,
                        }),
                    });
                }

                let turn_complete = game.apply_action(choice.action);

                self.sink.emit(RunnerEvent {
                    kind: RunnerEventKind::ActionApplied {
                        action: choice.action,
                    },
                    context: Some(RunnerEventContext {
                        game_number,
                        game: game.clone(),
                        turn_number,
                        turn,
                    }),
                });

                if let Some(max_turns) = self.max_turns
                    && turn_number > max_turns
                {
                    self.sink.emit(RunnerEvent {
                        kind: RunnerEventKind::GameFinished {
                            outcome: Outcome::Draw,
                        },
                        context: Some(RunnerEventContext {
                            game_number,
                            game: game.clone(),
                            turn_number,
                            turn,
                        }),
                    });

                    break;
                }

                match game.outcome() {
                    Outcome::InProgress => {}
                    outcome => {
                        self.sink.emit(RunnerEvent {
                            kind: RunnerEventKind::GameFinished { outcome },
                            context: Some(RunnerEventContext {
                                game_number,
                                game: game.clone(),
                                turn_number,
                                turn,
                            }),
                        });

                        break;
                    }
                }

                if turn_complete {
                    self.sink.emit(RunnerEvent {
                        kind: RunnerEventKind::TurnFinished,
                        context: Some(RunnerEventContext {
                            game_number,
                            game: game.clone(),
                            turn_number,
                            turn,
                        }),
                    });

                    game.end_turn();

                    turn = turn.advance();
                    turn_number += 1;

                    self.sink.emit(RunnerEvent {
                        kind: RunnerEventKind::TurnStarted,
                        context: Some(RunnerEventContext {
                            game_number,
                            game: game.clone(),
                            turn_number,
                            turn,
                        }),
                    });
                }
            }
        }

        self.sink.emit(RunnerEvent {
            kind: RunnerEventKind::RunnerFinished,
            context: None,
        });
    }
}

pub struct RunnerEvent<G: Game> {
    pub kind: RunnerEventKind<G>,
    pub context: Option<RunnerEventContext<G>>,
}

pub enum RunnerEventKind<G: Game> {
    RunnerStarted,
    GameStarted,
    TurnStarted,
    PositionEvaluated { evaluation: Evaluation<G> },
    ActionApplied { action: G::Action },
    TurnFinished,
    GameFinished { outcome: Outcome },
    RunnerFinished,
}

pub struct RunnerEventContext<G: Game> {
    pub game_number: u32,
    pub game: G,

    pub turn_number: u32,
    pub turn: Turn,
}
