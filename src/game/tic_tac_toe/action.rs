use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Place { index: u8 },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let square = match self {
            Action::Place { index } => match index {
                0 => "top-left".to_string(),
                1 => "top-center".to_string(),
                2 => "top-right".to_string(),
                3 => "middle-left".to_string(),
                4 => "middle-center".to_string(),
                5 => "middle-right".to_string(),
                6 => "bottom-left".to_string(),
                7 => "bottom-center".to_string(),
                8 => "bottom-right".to_string(),
                _ => unreachable!(),
            },
        };

        write!(f, "marks the {} square.", square)
    }
}
