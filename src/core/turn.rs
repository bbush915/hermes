#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Turn {
    Player,
    Opponent,
}

impl Turn {
    pub fn flip(self) -> Self {
        match self {
            Turn::Player => Turn::Opponent,
            Turn::Opponent => Turn::Player,
        }
    }
}
