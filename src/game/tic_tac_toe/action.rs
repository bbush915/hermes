#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Place { index: u8 },
}
