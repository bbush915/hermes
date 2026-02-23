#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Turn {
    Player1,
    Player2,
}

impl Turn {
    pub fn advance(self) -> Self {
        match self {
            Turn::Player1 => Turn::Player2,
            Turn::Player2 => Turn::Player1,
        }
    }
}
