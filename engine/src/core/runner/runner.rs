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
    threads: usize,

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
            threads: 1,

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

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads.max(1);

        self
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn run(&mut self)
    where
        G: Send,
        G::Action: Send,
        P1: Send,
        P2: Send,
    {
        #[cfg(not(target_arch = "wasm32"))]
        if self.threads > 1 {
            // self.run_parallel();
            return;
        }

        self.run_serial();
    }

    fn run_serial(&mut self) {
        self.sink.emit(RunnerEvent {
            kind: RunnerEventKind::RunnerStarted,
            context: None,
        });

        for game_number in 0..self.games {
            let initial_turn = if game_number % 2 == 0 {
                Turn::Player1
            } else {
                Turn::Player2
            };

            let events = run_single_game(
                game_number,
                initial_turn,
                &mut self.player_1,
                &mut self.player_2,
                self.max_turns,
            );

            for event in events {
                self.sink.emit(event);
            }
        }

        self.sink.emit(RunnerEvent {
            kind: RunnerEventKind::RunnerFinished,
            context: None,
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn run_parallel(&mut self)
    where
        G: Send,
        G::Action: Send,
        P1: Clone + Send,
        P2: Clone + Send,
    {
        use rayon::prelude::*;

        self.sink.emit(RunnerEvent {
            kind: RunnerEventKind::RunnerStarted,
            context: None,
        });

        let player_pairs: Vec<(P1, P2)> = (0..self.games)
            .map(|_| (self.player_1.clone(), self.player_2.clone()))
            .collect();

        let max_turns = self.max_turns;

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build()
            .expect("failed to build thread pool");

        let game_events: Vec<Vec<RunnerEvent<G>>> = pool.install(|| {
            player_pairs
                .into_par_iter()
                .enumerate()
                .map(|(game_number, (mut p1, mut p2))| {
                    let initial_turn = if game_number % 2 == 0 {
                        Turn::Player1
                    } else {
                        Turn::Player2
                    };

                    run_single_game(
                        game_number as u32,
                        initial_turn,
                        &mut p1,
                        &mut p2,
                        max_turns,
                    )
                })
                .collect()
        });

        for events in game_events {
            for event in events {
                self.sink.emit(event);
            }
        }

        self.sink.emit(RunnerEvent {
            kind: RunnerEventKind::RunnerFinished,
            context: None,
        });
    }
}

fn run_single_game<G, P1, P2>(
    game_number: u32,
    initial_turn: Turn,
    player_1: &mut P1,
    player_2: &mut P2,
    max_turns: Option<u32>,
) -> Vec<RunnerEvent<G>>
where
    G: Game,
    P1: Player<G>,
    P2: Player<G>,
{
    let mut events = vec![];

    let mut game = G::new();
    let mut turn_number = 0;
    let mut turn = initial_turn;

    events.push(RunnerEvent {
        kind: RunnerEventKind::GameStarted,
        context: Some(RunnerEventContext {
            game_number,
            game: game.clone(),
            turn_number,
            turn,
        }),
    });

    events.push(RunnerEvent {
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
            Turn::Player1 => player_1.choose_action(&game, turn_number),
            Turn::Player2 => player_2.choose_action(&game, turn_number),
        };

        if let Some(evaluation) = choice.evaluation {
            events.push(RunnerEvent {
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

        events.push(RunnerEvent {
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

        if let Some(max_turns) = max_turns
            && turn_number > max_turns
        {
            events.push(RunnerEvent {
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
                events.push(RunnerEvent {
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
            events.push(RunnerEvent {
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

            events.push(RunnerEvent {
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

    events
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
