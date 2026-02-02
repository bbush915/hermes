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
                let row = (*index / 6) + 1;
                let col = (*index % 6) + 1;

                write!(f, "places a {:?} at ({}, {}).", piece, row, col)
            }
            Action::Graduate { mask } => {
                write!(f, "graduates the piece(s) at ")?;

                let mut first = true;

                for i in 0..36 {
                    if (mask >> i) & 1 == 1 {
                        if !first {
                            write!(f, ", ")?;
                        }

                        let row = (i / 6) + 1;
                        let col = (i % 6) + 1;

                        write!(f, "({}, {})", row, col)?;

                        first = false;
                    }
                }

                write!(f, ".")?;

                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piece {
    Kitten,
    Cat,
}
