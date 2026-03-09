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
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    /// Creates edges from individual values in CSS order (top, right, bottom, left).
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            bottom,
            right,
            left,
        }
    }

    /// Returns edges where all sides are zero.
    pub fn zero() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            right: 0.0,
            left: 0.0,
        }
    }

    /// Returns edges where all four sides share the same value.
    pub fn all(value: f32) -> Self {
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
