#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Turn {
    PlayerOne,
    PlayerTwo,
}

impl Turn {
    pub fn advance(self) -> Self {
        match self {
            Turn::PlayerOne => Turn::PlayerTwo,
            Turn::PlayerTwo => Turn::PlayerOne,
        }
    }
}
