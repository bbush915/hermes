#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Place { piece: Piece, index: u8 },
    Graduate { mask: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piece {
    Kitten,
    Cat,
}
