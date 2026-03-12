pub mod column;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}
