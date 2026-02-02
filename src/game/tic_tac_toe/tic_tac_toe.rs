use std::iter::from_fn;
use std::mem::swap;
use std::{fmt, str};

use crate::core::{Game, Outcome};
use crate::game::tic_tac_toe::action::Action;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicTacToe {
    pub player_marks: u16,
    pub opponent_marks: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    player_marks: u16,
    opponent_marks: u16,
}

impl TicTacToe {
    pub const BOARD_SIZE: usize = 3;

    const BOARD_MASK: u16 = (1u16 << 9) - 1;
    const THREE_IN_A_ROW_MASKS: [u16; 8] = Self::make_three_in_a_row_masks();

    fn end_turn(&mut self) {
        self.flip_perspective();
    }

    fn into_indices(mut bits: u16) -> impl Iterator<Item = u8> {
        from_fn(move || {
            if bits == 0 {
                None
            } else {
                let mask = bits & (!bits + 1);
                bits ^= mask;

                Some(mask.trailing_zeros() as u8)
            }
        })
    }

    const fn make_three_in_a_row_masks() -> [u16; 8] {
        let mut masks = [0u16; 8];

        let mut n = 0;

        let mut x = 0;
        let mut y = 0;

        while x < Self::BOARD_SIZE {
            let is_x_lo = x < Self::BOARD_SIZE - 2;
            let is_x_hi = x > 1;

            while y < Self::BOARD_SIZE {
                let is_y_lo = y < Self::BOARD_SIZE - 2;

                // NOTE - Horizontal →

                if is_y_lo {
                    masks[n] = Self::xy_to_mask(x, y)
                        | Self::xy_to_mask(x, y + 1)
                        | Self::xy_to_mask(x, y + 2);

                    n += 1;
                }

                // NOTE - Vertical ↓

                if is_x_lo {
                    masks[n] = Self::xy_to_mask(x, y)
                        | Self::xy_to_mask(x + 1, y)
                        | Self::xy_to_mask(x + 2, y);

                    n += 1;
                }

                // NOTE - Diagonal ↙

                if is_x_hi && is_y_lo {
                    masks[n] = Self::xy_to_mask(x, y)
                        | Self::xy_to_mask(x - 1, y + 1)
                        | Self::xy_to_mask(x - 2, y + 2);

                    n += 1;
                }

                // NOTE - Diagonal ↘

                if is_x_lo && is_y_lo {
                    masks[n] = Self::xy_to_mask(x, y)
                        | Self::xy_to_mask(x + 1, y + 1)
                        | Self::xy_to_mask(x + 2, y + 2);

                    n += 1;
                }

                y += 1;
            }

            x += 1;
            y = 0;
        }

        masks
    }

    const fn xy_to_mask(x: usize, y: usize) -> u16 {
        1u16 << (x * Self::BOARD_SIZE + y)
    }
}

impl Game for TicTacToe {
    type Action = Action;
    type Checkpoint = Checkpoint;

    fn new() -> Self {
        TicTacToe {
            player_marks: 0,
            opponent_marks: 0,
        }
    }

    fn get_possible_actions(&self) -> Vec<Action> {
        if self.outcome() != Outcome::InProgress {
            return vec![];
        }

        let empty_squares = !(self.player_marks | self.opponent_marks) & Self::BOARD_MASK;
        let empty_square_count = empty_squares.count_ones() as usize;

        let mut actions = Vec::with_capacity(empty_square_count);

        for index in Self::into_indices(empty_squares) {
            actions.push(Action::Place { index });
        }

        actions
    }

    fn apply_action(&mut self, action: Action) -> bool {
        match action {
            Action::Place { index } => {
                let mask = 1u16 << index;

                self.player_marks |= mask;
            }
        };

        self.end_turn();

        true
    }

    fn outcome(&self) -> Outcome {
        // NOTE - Opponent

        for &mask in Self::THREE_IN_A_ROW_MASKS.iter() {
            if (self.opponent_marks & mask) == mask {
                return Outcome::Loss;
            }
        }

        // NOTE - Player

        for &mask in Self::THREE_IN_A_ROW_MASKS.iter() {
            if (self.player_marks & mask) == mask {
                return Outcome::Win;
            }
        }

        // NOTE - Draw

        if (self.player_marks | self.opponent_marks) & Self::BOARD_MASK == Self::BOARD_MASK {
            return Outcome::Draw;
        }

        Outcome::InProgress
    }

    fn create_checkpoint(&self) -> Checkpoint {
        Checkpoint {
            player_marks: self.player_marks,
            opponent_marks: self.opponent_marks,
        }
    }

    fn restore_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.player_marks = checkpoint.player_marks;
        self.opponent_marks = checkpoint.opponent_marks;
    }

    fn flip_perspective(&mut self) {
        swap(&mut self.player_marks, &mut self.opponent_marks);
    }
}

impl fmt::Display for TicTacToe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE - Board

        writeln!(formatter)?;
        writeln!(formatter, "╔═══╤═══╤═══╗")?;

        for x in 0..Self::BOARD_SIZE {
            write!(formatter, "║")?;

            for y in 0..Self::BOARD_SIZE {
                let mask = Self::xy_to_mask(x, y);

                let character = if self.player_marks & mask != 0 {
                    'X'
                } else if self.opponent_marks & mask != 0 {
                    'O'
                } else {
                    ' '
                };

                write!(formatter, " {} ", character)?;

                if y < Self::BOARD_SIZE - 1 {
                    write!(formatter, "│")?;
                }
            }

            writeln!(formatter, "║")?;

            if x < Self::BOARD_SIZE - 1 {
                writeln!(formatter, "╟───┼───┼───╢")?;
            }
        }

        writeln!(formatter, "╚═══╧═══╧═══╝")?;

        Ok(())
    }
}

impl str::FromStr for TicTacToe {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut player_marks = 0u16;
        let mut opponent_marks = 0u16;

        let lines: Vec<&str> = s.lines().collect();

        if lines.len() != 2 * Self::BOARD_SIZE + 1 {
            return Err("unexpected number of lines".to_string());
        }

        let board_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains('═') && !line.contains('─'))
            .copied()
            .collect();

        for (x, line) in board_lines.iter().enumerate() {
            let characters: Vec<char> = line
                .chars()
                .filter(|&character| character != '║' && character != '│')
                .collect();

            for y in 0..Self::BOARD_SIZE {
                let character = characters.get(y * 3 + 1).copied().unwrap_or(' ');

                if character == ' ' {
                    continue;
                }

                let mask = TicTacToe::xy_to_mask(x, y);

                match character {
                    'X' => player_marks |= mask,
                    'O' => opponent_marks |= mask,
                    _ => return Err(format!("invalid character: {}", character)),
                }
            }
        }

        Ok(TicTacToe {
            player_marks,
            opponent_marks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_game(value: &str) -> TicTacToe {
        value
            .trim()
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
            .parse()
            .unwrap()
    }

    trait TicTacToeTestExtensions {
        fn with_flipped_perspective(self) -> Self;
    }

    impl TicTacToeTestExtensions for TicTacToe {
        fn with_flipped_perspective(mut self) -> Self {
            self.flip_perspective();

            self
        }
    }

    fn xy_to_index(x: usize, y: usize) -> u8 {
        (x * TicTacToe::BOARD_SIZE + y) as u8
    }

    mod get_possible_actions {}

    mod apply_action {
        use super::*;

        #[test]
        fn should_place_mark() {
            let mut game = parse_game(
                "
                        ╔═══╤═══╤═══╗
                        ║   │   │   │
                        ╟───┼───┼───╢
                        ║   │   │   ║
                        ╟───┼───┼───╢
                        ║   │   │   ║
                        ╚═══╧═══╧═══╝
                    ",
            );

            game.apply_action(Action::Place {
                index: xy_to_index(0, 0),
            });

            let expected_game = parse_game(
                "
                        ╔═══╤═══╤═══╗
                        ║ X │   │   ║
                        ╟───┼───┼───╢
                        ║   │   │   ║
                        ╟───┼───┼───╢
                        ║   │   │   ║
                        ╚═══╧═══╧═══╝
                    ",
            )
            .with_flipped_perspective();

            assert_eq!(game, expected_game);
        }
    }

    mod create_checkpoint {}

    mod restore_checkpoint {}

    mod outcome {
        use super::*;

        #[test]
        fn should_detect_in_progress() {
            let game = parse_game(
                "
                    ╔═══╤═══╤═══╗
                    ║ X │   │   ║
                    ╟───┼───┼───╢
                    ║   │ O │   ║
                    ╟───┼───┼───╢
                    ║   │   │   ║
                    ╚═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::InProgress);
        }

        #[test]
        fn should_detect_horizontal_win() {
            let game = parse_game(
                "
                    ╔═══╤═══╤═══╗
                    ║ X │ X │ X ║
                    ╟───┼───┼───╢
                    ║   │   │   ║
                    ╟─-─┼───┼───╢
                    ║ O │   │ O ║
                    ╚═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_vertical_win() {
            let game = parse_game(
                "
                    ╔═══╤═══╤═══╗
                    ║ X │   │ O ║
                    ╟───┼───┼───╢
                    ║ X │   │   ║
                    ╟───┼───┼───╢
                    ║ X │   │ O ║
                    ╚═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_diagonal_left_win() {
            let game = parse_game(
                "
                    ╔═══╤═══╤═══╗
                    ║ O │   │ X ║
                    ╟───┼───┼───╢
                    ║   │ X │   ║
                    ╟───┼───┼───╢
                    ║ X │   │ O ║
                    ╚═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_diagonal_right_win() {
            let game = parse_game(
                "
                    ╔═══╤═══╤═══╗
                    ║ X │   │ O ║
                    ╟───┼───┼───╢
                    ║   │ X │   ║
                    ╟───┼───┼───╢
                    ║ O │   │ X ║
                    ╚═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }
    }
}
