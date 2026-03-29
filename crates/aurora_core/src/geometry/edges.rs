#![doc = include_str!("../../../../.wiki/Edges.md")]

use crate::geometry::size::Size;

/// Edge insets (top, right, bottom, left) describing spacing around a rectangle.
///
/// Used to represent padding, margins, or border widths. Follows CSS ordering:
/// top, right, bottom, left.
///
/// # Examples
///
/// ```
/// use aurora_core::geometry::edges::Edges;
///
/// // Uniform 10px padding on all sides
/// let padding = Edges::all(10.0);
///
/// // 8px horizontal, 16px vertical
/// let margin = Edges::symmetric(8.0, 16.0);
///
/// assert_eq!(margin.horizontal(), 16.0);
/// assert_eq!(margin.vertical(), 32.0);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    /// Creates edges from individual values in CSS order (top, right, bottom, left).
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            bottom,
            right,
            left,
        }
    }

    /// Returns edges where all sides are zero.
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            right: 0.0,
            left: 0.0,
        }
    }

    /// Returns edges where all four sides share the same value.
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            right: value,
            left: value,
        }
    }

    /// Alias for [`symmetric`](Self::symmetric).
    pub fn xy(x: f32, y: f32) -> Self {
        Self::symmetric(x, y)
    }

    /// Returns edges with symmetric horizontal and vertical values.
    ///
    /// `horizontal` is applied to left and right; `vertical` is applied to top and bottom.
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            right: horizontal,
            left: horizontal,
        }
    }

    /// Returns the total size occupied by these edges as a [`Size`].
    pub fn size(&self) -> Size {
        Size {
            width: self.horizontal(),
            height: self.vertical(),
        }
    }

    /// Returns the combined left and right edge widths.
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Returns the combined top and bottom edge widths.
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    /// Returns `true` if the horizontal pair and vertical pair are each equal.
    pub fn is_uniform(&self) -> bool {
        self.top == self.bottom && self.left == self.right
    }

    /// Returns `true` if all edges are zero.
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.bottom == 0.0 && self.left == 0.0 && self.right == 0.0
    }
}

impl From<f32> for Edges {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_fields() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.top, 1.0);
        assert_eq!(e.right, 2.0);
        assert_eq!(e.bottom, 3.0);
        assert_eq!(e.left, 4.0);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Edges::default(), Edges::zero());
        assert!(Edges::default().is_zero());
    }

    #[test]
    fn all_uniform() {
        let e = Edges::all(10.0);
        assert_eq!(e.top, 10.0);
        assert_eq!(e.right, 10.0);
        assert_eq!(e.bottom, 10.0);
        assert_eq!(e.left, 10.0);
    }

    #[test]
    fn symmetric_and_xy() {
        let e = Edges::symmetric(5.0, 10.0);
        assert_eq!(e.left, 5.0);
        assert_eq!(e.right, 5.0);
        assert_eq!(e.top, 10.0);
        assert_eq!(e.bottom, 10.0);
        assert_eq!(e, Edges::xy(5.0, 10.0));
    }

    #[test]
    fn horizontal_vertical() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.horizontal(), 6.0); // left + right
        assert_eq!(e.vertical(), 4.0); // top + bottom
    }

    #[test]
    fn size_returns_total() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        let s = e.size();
        assert_eq!(s.width, 6.0);
        assert_eq!(s.height, 4.0);
    }

    #[test]
    fn is_uniform_checks_pairs() {
        assert!(Edges::all(5.0).is_uniform());
        assert!(Edges::symmetric(3.0, 7.0).is_uniform());
        assert!(!Edges::new(1.0, 2.0, 3.0, 4.0).is_uniform());
    }

    #[test]
    fn is_zero_checks_all() {
        assert!(Edges::zero().is_zero());
        assert!(!Edges::new(0.0, 0.0, 0.0, 1.0).is_zero());
    }

    #[test]
    fn from_f32() {
        let e: Edges = 8.0.into();
        assert_eq!(e, Edges::all(8.0));
    }
}
