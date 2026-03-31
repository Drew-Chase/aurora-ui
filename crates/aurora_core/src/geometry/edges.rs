#![doc = include_str!("../../../../.wiki/Edges.md")]

use crate::direction::TextDirection;
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

    /// Returns the inline-start edge (`left` in LTR, `right` in RTL).
    pub fn inline_start(&self, direction: TextDirection) -> f32 {
        match direction {
            TextDirection::Ltr => self.left,
            TextDirection::Rtl => self.right,
        }
    }

    /// Returns the inline-end edge (`right` in LTR, `left` in RTL).
    pub fn inline_end(&self, direction: TextDirection) -> f32 {
        match direction {
            TextDirection::Ltr => self.right,
            TextDirection::Rtl => self.left,
        }
    }

    /// Returns the block-start edge (always `top` in horizontal writing modes).
    pub fn block_start(&self) -> f32 {
        self.top
    }

    /// Returns the block-end edge (always `bottom` in horizontal writing modes).
    pub fn block_end(&self) -> f32 {
        self.bottom
    }

    /// Returns a copy with left and right swapped when the direction is RTL.
    ///
    /// In LTR mode, returns `self` unchanged. Use this to convert padding or
    /// margin that was specified with physical values into direction-aware values.
    pub fn resolve(&self, direction: TextDirection) -> Self {
        match direction {
            TextDirection::Ltr => *self,
            TextDirection::Rtl => Self {
                top: self.top,
                right: self.left,
                bottom: self.bottom,
                left: self.right,
            },
        }
    }

    /// Creates edges from logical values, resolving to physical edges based on direction.
    ///
    /// In LTR: `inline_start` → `left`, `inline_end` → `right`.
    /// In RTL: `inline_start` → `right`, `inline_end` → `left`.
    pub fn logical(
        block_start: f32,
        inline_end: f32,
        block_end: f32,
        inline_start: f32,
        direction: TextDirection,
    ) -> Self {
        match direction {
            TextDirection::Ltr => Self::new(block_start, inline_end, block_end, inline_start),
            TextDirection::Rtl => Self::new(block_start, inline_start, block_end, inline_end),
        }
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
    use crate::direction::TextDirection;

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

    #[test]
    fn inline_start_ltr() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.inline_start(TextDirection::Ltr), 4.0); // left
    }

    #[test]
    fn inline_start_rtl() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.inline_start(TextDirection::Rtl), 2.0); // right
    }

    #[test]
    fn inline_end_ltr() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.inline_end(TextDirection::Ltr), 2.0); // right
    }

    #[test]
    fn inline_end_rtl() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.inline_end(TextDirection::Rtl), 4.0); // left
    }

    #[test]
    fn block_start_and_end() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.block_start(), 1.0);
        assert_eq!(e.block_end(), 3.0);
    }

    #[test]
    fn resolve_ltr_unchanged() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.resolve(TextDirection::Ltr), e);
    }

    #[test]
    fn resolve_rtl_swaps_left_right() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        let resolved = e.resolve(TextDirection::Rtl);
        assert_eq!(resolved.top, 1.0);
        assert_eq!(resolved.right, 4.0); // was left
        assert_eq!(resolved.bottom, 3.0);
        assert_eq!(resolved.left, 2.0); // was right
    }

    #[test]
    fn logical_ltr() {
        let e = Edges::logical(1.0, 2.0, 3.0, 4.0, TextDirection::Ltr);
        assert_eq!(e.top, 1.0);
        assert_eq!(e.right, 2.0); // inline_end
        assert_eq!(e.bottom, 3.0);
        assert_eq!(e.left, 4.0); // inline_start
    }

    #[test]
    fn logical_rtl() {
        let e = Edges::logical(1.0, 2.0, 3.0, 4.0, TextDirection::Rtl);
        assert_eq!(e.top, 1.0);
        assert_eq!(e.right, 4.0); // inline_start (flipped)
        assert_eq!(e.bottom, 3.0);
        assert_eq!(e.left, 2.0); // inline_end (flipped)
    }
}
