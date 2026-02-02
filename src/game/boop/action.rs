use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Place { piece: Piece, index: u8 },
    Graduate { mask: u64 },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Place { piece, index } => {
                let col = (*index % 6) as u8 + b'A';
                let row = (*index / 6) + 1;

                writeln!(f, "places {:?} at {}{}", piece, col as char, row)
            }
            Action::Graduate { mask } => {
                write!(f, "graduates ")?;

                let mut first = true;

                for i in 0..36 {
                    if (mask >> i) & 1 == 1 {
                        if !first {
                            write!(f, ", ")?;
                        }

                        let col = (i % 6) as u8 + b'A';
                        let row = (i / 6) + 1;

                        write!(f, "{}{}", col as char, row)?;

                        first = false;
                    }
                }
                writeln!(f)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piece {
    Kitten,
    Cat,
}
