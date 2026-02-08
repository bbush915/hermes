use std::iter::from_fn;
use std::mem::swap;
use std::{fmt, str};

use crate::core::{Game, Outcome, Turn};
use crate::game::boop::action::{Action, Piece};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Boop {
    pub phase: Phase,

    pub player_cats: u64,
    pub player_kittens: u64,
    pub player_graduations: u8,

    pub opponent_cats: u64,
    pub opponent_kittens: u64,
    pub opponent_graduations: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Place,
    Graduate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    phase: Phase,

    player_cats: u64,
    player_kittens: u64,
    player_graduations: u8,

    opponent_cats: u64,
    opponent_kittens: u64,
    opponent_graduations: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pool {
    pub kittens_available: u8,
    pub cats_available: u8,
}

impl Boop {
    pub const BOARD_SIZE: usize = 6;
    pub const POOL_SIZE: u8 = 8;

    const BOARD_MASK: u64 = (1u64 << 36) - 1;
    const NEIGHBOR_MASKS: [u64; Self::BOARD_SIZE * Self::BOARD_SIZE] = Self::make_neighbor_masks();
    pub const THREE_IN_A_ROW_MASKS: [u64; 80] = Self::make_three_in_a_row_masks();

    pub fn player_pool(&self) -> Pool {
        let kittens_played = u8::try_from(self.player_kittens.count_ones()).unwrap();
        let kittens_available = Self::POOL_SIZE - self.player_graduations - kittens_played;

        let cats_played = u8::try_from(self.player_cats.count_ones()).unwrap();
        let cats_available = self.player_graduations - cats_played;

        Pool {
            kittens_available,
            cats_available,
        }
    }

    pub fn opponent_pool(&self) -> Pool {
        let kittens_played = u8::try_from(self.opponent_kittens.count_ones()).unwrap();
        let kittens_available = Self::POOL_SIZE - self.opponent_graduations - kittens_played;

        let cats_played = u8::try_from(self.opponent_cats.count_ones()).unwrap();
        let cats_available = self.opponent_graduations - cats_played;

        Pool {
            kittens_available,
            cats_available,
        }
    }

    fn get_possible_place_actions(&self) -> Vec<Action> {
        debug_assert!(self.phase == Phase::Place);

        let Pool {
            kittens_available,
            cats_available,
        } = self.player_pool();

        let has_kitten_available = kittens_available > 0;
        let has_cat_available = cats_available > 0;

        let empty_squares =
            !(self.player_kittens | self.player_cats | self.opponent_kittens | self.opponent_cats)
                & Self::BOARD_MASK;
        let empty_square_count = empty_squares.count_ones() as usize;

        let mut actions = Vec::with_capacity(
            empty_square_count
                * (usize::from(has_kitten_available) + usize::from(has_cat_available)),
        );

        for index in Self::into_indices(empty_squares) {
            if has_kitten_available {
                actions.push(Action::Place {
                    piece: Piece::Kitten,
                    index,
                });
            }

            if has_cat_available {
                actions.push(Action::Place {
                    piece: Piece::Cat,
                    index,
                });
            }
        }

        actions
    }

    fn get_possible_graduate_actions(&self) -> Vec<Action> {
        let mut player_pieces = self.player_kittens | self.player_cats;

        let mut actions =
            Vec::with_capacity(Self::THREE_IN_A_ROW_MASKS.len() + Self::POOL_SIZE as usize);

        for &mask in &Self::THREE_IN_A_ROW_MASKS {
            if (player_pieces & mask) == mask {
                actions.push(Action::Graduate { mask });
            }
        }

        if u8::try_from(player_pieces.count_ones()).unwrap() == Self::POOL_SIZE {
            while player_pieces != 0 {
                let mask = player_pieces & (!player_pieces + 1);
                player_pieces &= player_pieces - 1;

                actions.push(Action::Graduate { mask });
            }
        }

        actions
    }

    fn apply_place_action(&mut self, piece: Piece, index: u8) {
        let mask = 1u64 << index;

        match piece {
            Piece::Kitten => self.player_kittens |= mask,
            Piece::Cat => self.player_cats |= mask,
        }

        self.apply_boop(piece, index);
    }

    fn apply_boop(&mut self, piece: Piece, index: u8) {
        let all_pieces =
            self.player_kittens | self.player_cats | self.opponent_kittens | self.opponent_cats;

        let boopable_pieces = match piece {
            Piece::Kitten => self.player_kittens | self.opponent_kittens,
            Piece::Cat => all_pieces,
        };

        let (x, y) = Self::index_to_xy(index);

        let mut neighbor_pieces = boopable_pieces & Self::NEIGHBOR_MASKS[index as usize];

        while neighbor_pieces != 0 {
            let mask = neighbor_pieces & (!neighbor_pieces + 1);
            neighbor_pieces &= neighbor_pieces - 1;

            let target_board = if self.player_kittens & mask != 0 {
                &mut self.player_kittens
            } else if self.player_cats & mask != 0 {
                &mut self.player_cats
            } else if self.opponent_kittens & mask != 0 {
                &mut self.opponent_kittens
            } else {
                &mut self.opponent_cats
            };

            let adjacent_index = u8::try_from(mask.trailing_zeros()).unwrap();
            let (adj_x, adj_y) = Self::index_to_xy(adjacent_index);

            let dx = adj_x.cast_signed() - x.cast_signed();
            let dy = adj_y.cast_signed() - y.cast_signed();

            let x_ = adj_x.cast_signed() + dx;
            let y_ = adj_y.cast_signed() + dy;

            if x_ < 0
                || x_ >= Self::BOARD_SIZE.cast_signed()
                || y_ < 0
                || y_ >= Self::BOARD_SIZE.cast_signed()
            {
                *target_board &= !mask;
                continue;
            }

            let destination_mask = Self::xy_to_mask(x_.cast_unsigned(), y_.cast_unsigned());

            if (destination_mask & all_pieces) != 0 {
                continue;
            }

            *target_board &= !mask;
            *target_board |= destination_mask;
        }
    }

    fn apply_graduate_action(&mut self, mask: u64) {
        let kittens_removed = u8::try_from((self.player_kittens & mask).count_ones()).unwrap();

        self.player_kittens &= !mask;
        self.player_cats &= !mask;

        self.player_graduations += kittens_removed;
    }

    fn flip_perspective(&mut self) {
        swap(&mut self.player_kittens, &mut self.opponent_kittens);
        swap(&mut self.player_cats, &mut self.opponent_cats);
        swap(&mut self.player_graduations, &mut self.opponent_graduations);
    }

    fn into_indices(mut bits: u64) -> impl Iterator<Item = u8> {
        from_fn(move || {
            if bits == 0 {
                None
            } else {
                let mask = bits & (!bits + 1);
                bits ^= mask;

                Some(u8::try_from(mask.trailing_zeros()).unwrap())
            }
        })
    }

    fn index_to_xy(index: u8) -> (usize, usize) {
        let x = (index as usize) / Self::BOARD_SIZE;
        let y = (index as usize) % Self::BOARD_SIZE;

        (x, y)
    }

    const fn make_neighbor_masks() -> [u64; Self::BOARD_SIZE * Self::BOARD_SIZE] {
        let mut masks = [0u64; Self::BOARD_SIZE * Self::BOARD_SIZE];

        let mut n = 0;

        let mut x = 0;
        let mut y = 0;

        while x < Self::BOARD_SIZE {
            while y < Self::BOARD_SIZE {
                let mut mask = 0u64;

                let mut dx: isize = -1;
                let mut dy: isize = -1;

                while dx <= 1 {
                    while dy <= 1 {
                        if dx == 0 && dy == 0 {
                            dy += 1;
                            continue;
                        }

                        let x_ = x.cast_signed() + dx;
                        let y_ = y.cast_signed() + dy;

                        if x_ >= 0
                            && x_ < Self::BOARD_SIZE.cast_signed()
                            && y_ >= 0
                            && y_ < Self::BOARD_SIZE.cast_signed()
                        {
                            mask |= Self::xy_to_mask(x_.cast_unsigned(), y_.cast_unsigned());
                        }

                        dy += 1;
                    }

                    dx += 1;
                    dy = -1;
                }

                masks[n] = mask;

                n += 1;

                y += 1;
            }

            x += 1;
            y = 0;
        }

        masks
    }

    const fn make_three_in_a_row_masks() -> [u64; 80] {
        let mut masks = [0u64; 80];

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

    const fn xy_to_mask(x: usize, y: usize) -> u64 {
        1u64 << (x * Self::BOARD_SIZE + y)
    }
}

impl Game for Boop {
    type Action = Action;
    type Checkpoint = Checkpoint;

    fn new() -> Self {
        Boop {
            phase: Phase::Place,

            player_cats: 0,
            player_kittens: 0,
            player_graduations: 0,

            opponent_cats: 0,
            opponent_kittens: 0,
            opponent_graduations: 0,
        }
    }

    fn outcome(&self) -> Outcome {
        // NOTE - Opponent

        for &mask in &Self::THREE_IN_A_ROW_MASKS {
            if (self.opponent_cats & mask) == mask {
                return Outcome::Loss;
            }
        }

        if u8::try_from(self.opponent_cats.count_ones()).unwrap() == Self::POOL_SIZE {
            return Outcome::Loss;
        }

        // NOTE - Player

        for &mask in &Self::THREE_IN_A_ROW_MASKS {
            if (self.player_cats & mask) == mask {
                return Outcome::Win;
            }
        }

        if u8::try_from(self.player_cats.count_ones()).unwrap() == Self::POOL_SIZE {
            return Outcome::Win;
        }

        Outcome::InProgress
    }

    fn get_possible_actions(&self) -> Vec<Action> {
        if self.outcome() != Outcome::InProgress {
            return vec![];
        }

        match self.phase {
            Phase::Place => self.get_possible_place_actions(),
            Phase::Graduate => self.get_possible_graduate_actions(),
        }
    }

    fn apply_action(&mut self, action: Action) -> bool {
        let mut turn_complete = true;

        match action {
            Action::Place { piece, index } => {
                self.apply_place_action(piece, index);

                let graduate_actions = self.get_possible_graduate_actions();

                if !graduate_actions.is_empty() {
                    self.phase = Phase::Graduate;

                    turn_complete = false;
                }
            }
            Action::Graduate { mask } => {
                self.apply_graduate_action(mask);
            }
        }

        turn_complete
    }

    fn end_turn(&mut self) {
        self.phase = Phase::Place;

        self.flip_perspective();
    }

    fn create_checkpoint(&self) -> Checkpoint {
        Checkpoint {
            phase: self.phase,

            player_kittens: self.player_kittens,
            player_cats: self.player_cats,
            player_graduations: self.player_graduations,

            opponent_kittens: self.opponent_kittens,
            opponent_cats: self.opponent_cats,
            opponent_graduations: self.opponent_graduations,
        }
    }

    fn restore_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.phase = checkpoint.phase;

        self.player_kittens = checkpoint.player_kittens;
        self.player_cats = checkpoint.player_cats;
        self.player_graduations = checkpoint.player_graduations;

        self.opponent_kittens = checkpoint.opponent_kittens;
        self.opponent_cats = checkpoint.opponent_cats;
        self.opponent_graduations = checkpoint.opponent_graduations;
    }

    fn display(&self, turn: Turn) -> String {
        let mut game = self.clone();

        if turn == Turn::PlayerTwo {
            game.flip_perspective();
        }

        format!("{game}")
    }
}

impl fmt::Display for Boop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE - Player

        let player_pool = self.player_pool();

        write!(formatter, "Player: ")?;

        for _ in 0..player_pool.cats_available {
            write!(formatter, "X ")?;
        }

        for _ in 0..player_pool.kittens_available {
            write!(formatter, "x ")?;
        }

        writeln!(formatter)?;

        // NOTE - Opponent

        let opponent_pool = self.opponent_pool();

        write!(formatter, "Opponent: ")?;

        for _ in 0..opponent_pool.cats_available {
            write!(formatter, "O ")?;
        }

        for _ in 0..opponent_pool.kittens_available {
            write!(formatter, "o ")?;
        }

        writeln!(formatter)?;

        // NOTE - Board

        writeln!(formatter, "╔═══╤═══╤═══╤═══╤═══╤═══╗")?;

        for x in 0..Self::BOARD_SIZE {
            write!(formatter, "║")?;

            for y in 0..Self::BOARD_SIZE {
                let mask = Self::xy_to_mask(x, y);

                let character = if self.player_cats & mask != 0 {
                    'X'
                } else if self.player_kittens & mask != 0 {
                    'x'
                } else if self.opponent_cats & mask != 0 {
                    'O'
                } else if self.opponent_kittens & mask != 0 {
                    'o'
                } else {
                    ' '
                };

                write!(formatter, " {character} ")?;

                if y < Self::BOARD_SIZE - 1 {
                    write!(formatter, "│")?;
                }
            }

            writeln!(formatter, "║")?;

            if x < Self::BOARD_SIZE - 1 {
                writeln!(formatter, "╟───┼───┼───┼───┼───┼───╢")?;
            }
        }

        writeln!(formatter, "╚═══╧═══╧═══╧═══╧═══╧═══╝")?;

        Ok(())
    }
}

impl str::FromStr for Boop {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut player_kittens = 0u64;
        let mut player_cats = 0u64;

        let mut player_kittens_played = 0u8;
        let mut player_kittens_available = 0u8;
        let mut player_cats_played = 0u8;
        let mut player_cats_available = 0u8;
        let mut player_graduations = 0u8;

        let mut opponent_kittens = 0u64;
        let mut opponent_cats = 0u64;

        let mut opponent_kittens_played = 0u8;
        let mut opponent_kittens_available = 0u8;
        let mut opponent_cats_played = 0u8;
        let mut opponent_cats_available = 0u8;
        let mut opponent_graduations = 0u8;

        let lines: Vec<&str> = s.lines().collect();

        if lines.len() != 2 * Self::BOARD_SIZE + 4 {
            return Err("unexpected number of lines".to_string());
        }

        if let Some(player_line) = lines.first() {
            if player_line.starts_with("Player:") {
                let pool = player_line.trim_start_matches("Player:").trim();

                player_kittens_available = u8::try_from(pool.matches('x').count()).unwrap();
                player_cats_available = u8::try_from(pool.matches('X').count()).unwrap();

                player_graduations = player_cats_available;
            } else {
                return Err("missing player pool".to_string());
            }
        }

        if let Some(opponent_line) = lines.get(1) {
            if opponent_line.starts_with("Opponent:") {
                let pool = opponent_line.trim_start_matches("Opponent:").trim();

                opponent_kittens_available = u8::try_from(pool.matches('o').count()).unwrap();
                opponent_cats_available = u8::try_from(pool.matches('O').count()).unwrap();

                opponent_graduations = opponent_cats_available;
            } else {
                return Err("missing opponent pool".to_string());
            }
        }

        let board_lines: Vec<&str> = lines
            .iter()
            .skip(3)
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

                let mask = Boop::xy_to_mask(x, y);

                match character {
                    'X' => {
                        player_cats |= mask;

                        player_cats_played += 1;

                        player_graduations += 1;
                    }
                    'x' => {
                        player_kittens |= mask;

                        player_kittens_played += 1;
                    }
                    'O' => {
                        opponent_cats |= mask;

                        opponent_cats_played += 1;

                        opponent_graduations += 1;
                    }
                    'o' => {
                        opponent_kittens |= mask;

                        opponent_kittens_played += 1;
                    }
                    _ => return Err(format!("invalid character: {character}")),
                }
            }
        }

        assert_eq!(
            player_kittens_played
                + player_kittens_available
                + player_cats_played
                + player_cats_available,
            Self::POOL_SIZE
        );

        assert_eq!(
            player_cats_available + player_cats_played,
            player_graduations
        );

        assert_eq!(
            opponent_kittens_played
                + opponent_kittens_available
                + opponent_cats_played
                + opponent_cats_available,
            Self::POOL_SIZE
        );

        assert_eq!(
            opponent_cats_available + opponent_cats_played,
            opponent_graduations
        );

        Ok(Boop {
            phase: Phase::Place,

            player_kittens,
            player_cats,
            player_graduations,

            opponent_kittens,
            opponent_cats,
            opponent_graduations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_game(value: &str) -> Boop {
        value
            .trim()
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .parse()
            .expect("unable to parse game")
    }

    trait BoopTestExtensions {
        fn with_phase(self, phase: Phase) -> Self;
    }

    impl BoopTestExtensions for Boop {
        fn with_phase(mut self, phase: Phase) -> Self {
            self.phase = phase;

            self
        }
    }

    fn xy_to_index(x: usize, y: usize) -> u8 {
        u8::try_from(x * Boop::BOARD_SIZE + y).unwrap()
    }

    fn xys_to_mask(xys: &[(usize, usize)]) -> u64 {
        let mut mask = 0u64;

        for &(x, y) in xys {
            mask |= 1u64 << xy_to_index(x, y);
        }

        mask
    }

    mod get_possible_actions {}

    mod apply_action {
        use super::*;

        mod apply_place_action {
            use super::*;

            #[test]
            fn should_place_kitten() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(0, 0),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║ x │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_place_cat() {
                let mut game = parse_game(
                    "
                        Player: X x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Cat,
                    index: xy_to_index(0, 0),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║ X │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn kitten_should_boop_kitten() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ o │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(2, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │   │ o │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn kitten_should_not_boop_cat() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ O │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(2, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │ O │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn cat_should_boop_kitten() {
                let mut game = parse_game(
                    "
                        Player: X x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ o │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Cat,
                    index: xy_to_index(2, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ X │   │ o │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn cat_should_boop_cat() {
                let mut game = parse_game(
                    "
                        Player: X x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ O │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Cat,
                    index: xy_to_index(2, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ X │   │ O │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_not_boop_if_blocked() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ o │ o │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(2, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │ o │ o │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_top_left_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║ o │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(1, 1),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_top_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │ o │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(1, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_top_right_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │ o ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(1, 4),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_left_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║ o │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(3, 1),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_right_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │ o ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(2, 4),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_bottom_left_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║ o │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(4, 1),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_bottom_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ o │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(4, 3),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ x │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_handle_bottom_right_removal() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x x x
                        Opponent: o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │ o ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(4, 4),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                assert_eq!(game, expected_game);
            }

            #[test]
            fn should_transition_to_graduate_phase_if_available() {
                let mut game = parse_game(
                    "
                        Player: x x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │ x │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                );

                game.apply_action(Action::Place {
                    piece: Piece::Kitten,
                    index: xy_to_index(2, 2),
                });

                let expected_game = parse_game(
                    "
                        Player: x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │ x │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                assert_eq!(game, expected_game);
            }
        }

        mod apply_graduate_action {
            use super::*;

            #[test]
            fn can_graduate_all_kittens() {
                let mut game = parse_game(
                    "
                        Player: x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │ x │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                game.apply_action(Action::Graduate {
                    mask: xys_to_mask(&[(2, 2), (2, 3), (2, 4)]),
                });

                let expected_game = parse_game(
                    "
                        Player: X X X x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                assert_eq!(game, expected_game);
            }

            #[test]
            fn can_graduate_some_kittens() {
                let mut game = parse_game(
                    "
                        Player: x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │ X │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                game.apply_action(Action::Graduate {
                    mask: xys_to_mask(&[(2, 2), (2, 3), (2, 4)]),
                });

                let expected_game = parse_game(
                    "
                        Player: X X X x x x x x
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                assert_eq!(game, expected_game);
            }

            #[test]
            fn can_graduate_single_kitten() {
                let mut game = parse_game(
                    "
                        Player:
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │ x │   │ x │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │ x ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║ x │   │   │   │ x │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                game.apply_action(Action::Graduate {
                    mask: xys_to_mask(&[(0, 1)]),
                });

                let expected_game = parse_game(
                    "
                        Player: X
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │ x │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │ x ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║ x │   │   │   │ x │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                assert_eq!(game, expected_game);
            }

            #[test]
            fn can_graduate_single_cat() {
                let mut game = parse_game(
                    "
                        Player:
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │ X │   │ x │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │ x ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║ x │   │   │   │ x │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                game.apply_action(Action::Graduate {
                    mask: xys_to_mask(&[(0, 1)]),
                });

                let expected_game = parse_game(
                    "
                        Player: X
                        Opponent: o o o o o o o o

                        ╔═══╤═══╤═══╤═══╤═══╤═══╗
                        ║   │   │   │ x │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │   │ x ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │   │   │ x │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │ x │   │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║   │   │ x │   │   │   ║
                        ╟───┼───┼───┼───┼───┼───╢
                        ║ x │   │   │   │ x │   ║
                        ╚═══╧═══╧═══╧═══╧═══╧═══╝
                    ",
                )
                .with_phase(Phase::Graduate);

                assert_eq!(game, expected_game);
            }
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
                    Player: x x x x x x
                    Opponent: o o o o o o

                    ╔═══╤═══╤═══╤═══╤═══╤═══╗
                    ║ X │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │ x │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │ O │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │ o │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╚═══╧═══╧═══╧═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::InProgress);
        }

        #[test]
        fn should_detect_horizontal_win() {
            let game = parse_game(
                "
                    Player: x x x x x
                    Opponent: o o o o o o o o

                    ╔═══╤═══╤═══╤═══╤═══╤═══╗
                    ║ X │ X │ X │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╚═══╧═══╧═══╧═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_vertical_win() {
            let game = parse_game(
                "
                    Player: x x x x x
                    Opponent: o o o o o o o o

                    ╔═══╤═══╤═══╤═══╤═══╤═══╗
                    ║ X │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║ X │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║ X │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╚═══╧═══╧═══╧═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_diagonal_left_win() {
            let game = parse_game(
                "
                    Player: x x x x x
                    Opponent: o o o o o o o o

                    ╔═══╤═══╤═══╤═══╤═══╤═══╗
                    ║   │   │ X │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │ X │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║ X │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╚═══╧═══╧═══╧═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_diagonal_right_win() {
            let game = parse_game(
                "
                    Player: x x x x x
                    Opponent: o o o o o o o o

                    ╔═══╤═══╤═══╤═══╤═══╤═══╗
                    ║ X │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │ X │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │ X │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │   ║
                    ╚═══╧═══╧═══╧═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }

        #[test]
        fn should_detect_all_cats_win() {
            let game = parse_game(
                "
                    Player:
                    Opponent: o o o o o o o o

                    ╔═══╤═══╤═══╤═══╤═══╤═══╗
                    ║ X │   │   │   │ X │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │ X │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │   │ X ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │ X │   │   │   │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║   │   │   │   │ X │   ║
                    ╟───┼───┼───┼───┼───┼───╢
                    ║ X │   │   │ X │   │   ║
                    ╚═══╧═══╧═══╧═══╧═══╧═══╝
                ",
            );

            let outcome = game.outcome();

            assert_eq!(outcome, Outcome::Win);
        }
    }
}
