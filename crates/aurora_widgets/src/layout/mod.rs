/// Layout containers for arranging child widgets.
///
/// This module provides four layout primitives:
///
/// - [`Column`](column::Column) — vertical stack (use the [`col!`] macro)
/// - [`Row`](row::Row) — horizontal stack (use the [`row!`] macro)
/// - [`Stack`](stack::Stack) — overlapping layers (z-order)
/// - [`Positioned`](position::Positioned) — absolute/fixed positioning
///
/// Column and Row share [`Justify`] and [`Align`] enums for controlling
/// distribution along the main and cross axes respectively.
pub mod column;
pub mod row;
pub mod position;
pub mod stack;

/// Controls distribution of children along the main axis.
///
/// For [`Column`](column::Column) the main axis is vertical;
/// for [`Row`](row::Row) it is horizontal.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Justify {
    /// Pack children at the start of the main axis.
    Start,
    /// Center children in the available space.
    Center,
    /// Pack children at the end of the main axis.
    End,
    /// Distribute leftover space evenly between children.
    SpaceBetween,
}

/// Controls alignment of children along the cross axis.
///
/// For [`Column`](column::Column) the cross axis is horizontal;
/// for [`Row`](row::Row) it is vertical.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Align {
    /// Align children to the start of the cross axis.
    Start,
    /// Center children on the cross axis.
    Center,
    /// Align children to the end of the cross axis.
    End,
    /// Stretch children to fill the cross axis.
    Stretch,
}
