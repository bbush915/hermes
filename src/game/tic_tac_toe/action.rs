use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Place { index: u8 },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let square = match self {
            Action::Place { index } => match index {
                0 => "top-left",
                1 => "top-center",
                2 => "top-right",
                3 => "middle-left",
                4 => "middle-center",
                5 => "middle-right",
                6 => "bottom-left",
                7 => "bottom-center",
                8 => "bottom-right",
                _ => unreachable!(),
            },
        };

        write!(f, "marks the {} square.", square)
    }
}
