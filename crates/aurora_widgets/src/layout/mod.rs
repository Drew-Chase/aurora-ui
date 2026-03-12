pub mod column;
pub mod row;

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
