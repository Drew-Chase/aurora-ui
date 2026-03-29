#![doc = include_str!("../../../../.wiki/Corners.md")]

/// Corner radii for a rounded rectangle.
///
/// Stores independent radii for each corner in CSS order: top-left, top-right,
/// bottom-right, bottom-left.
///
/// Most methods come in two variants:
/// - **Immutable** (e.g. [`top`](Self::top)) — returns a new `Corners` with the change applied.
/// - **Mutable** (e.g. [`top_mut`](Self::top_mut)) — modifies in place and returns `&mut Self` for chaining.
///
/// # Examples
///
/// ```
/// use aurora_core::geometry::corners::Corners;
///
/// // Uniform 8px border radius
/// let corners = Corners::all(8.0);
///
/// // Only round the top corners
/// let top_only = Corners::zero().top(12.0);
///
/// // Chain mutable calls
/// let mut c = Corners::zero();
/// c.top_mut(8.0).left_mut(4.0);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    /// Creates corners from individual radii (top-left, top-right, bottom-right, bottom-left).
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// Returns corners where all four radii share the same value.
    pub const fn all(radius: f32) -> Self {
        Corners {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Returns corners with all radii set to zero.
    pub const fn zero() -> Self {
        Self::all(0.0)
    }

    /// Returns a new `Corners` with both top radii set to `radius`.
    pub fn top(&self, radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            ..*self
        }
    }

    /// Sets both top radii to `radius` in place.
    pub fn top_mut(&mut self, radius: f32) -> &mut Self {
        self.top_left = radius;
        self.top_right = radius;
        self
    }

    /// Returns a new `Corners` with both bottom radii set to `radius`.
    pub fn bottom(&self, radius: f32) -> Self {
        Self {
            bottom_left: radius,
            bottom_right: radius,
            ..*self
        }
    }

    /// Sets both bottom radii to `radius` in place.
    pub fn bottom_mut(&mut self, radius: f32) -> &mut Self {
        self.bottom_left = radius;
        self.bottom_right = radius;
        self
    }

    /// Returns a new `Corners` with both left radii set to `radius`.
    pub fn left(&self, radius: f32) -> Self {
        Self {
            top_left: radius,
            bottom_left: radius,
            ..*self
        }
    }

    /// Sets both left radii to `radius` in place.
    pub fn left_mut(&mut self, radius: f32) -> &mut Self {
        self.top_left = radius;
        self.bottom_left = radius;
        self
    }

    /// Returns a new `Corners` with both right radii set to `radius`.
    pub fn right(&self, radius: f32) -> Self {
        Self {
            top_right: radius,
            bottom_right: radius,
            ..*self
        }
    }

    /// Sets both right radii to `radius` in place.
    pub fn right_mut(&mut self, radius: f32) -> &mut Self {
        self.top_right = radius;
        self.bottom_right = radius;
        self
    }

    /// Returns a new `Corners` with all radii set to `radius`.
    pub fn set(&self, radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Sets all radii to `radius` in place.
    pub fn set_mut(&mut self, radius: f32) -> &mut Self {
        self.top_left = radius;
        self.top_right = radius;
        self.bottom_left = radius;
        self.bottom_right = radius;
        self
    }

    /// Alias for [`zero`](Self::zero). Returns corners with no rounding (sharp edges).
    pub fn square() -> Self {
        Self::all(0.0)
    }

    /// Returns `true` if all corners have zero radius (sharp rectangle).
    pub fn is_square(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_left == 0.0
            && self.bottom_right == 0.0
    }

    /// Converts the corners to an array `[top_left, top_right, bottom_right, bottom_left]`.
    pub fn to_array(&self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }

    /// Creates corners from an array `[top_left, top_right, bottom_right, bottom_left]`.
    pub fn from_array(array: &[f32; 4]) -> Self {
        Self {
            top_left: array[0],
            top_right: array[1],
            bottom_right: array[2],
            bottom_left: array[3],
        }
    }

    /// Returns `true` if all four radii are equal.
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }

    /// Returns `true` if all radii are zero.
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0 && self.top_right == 0.0 && self.bottom_left == 0.0 && self.bottom_right == 0.0
    }
}

impl From<f32> for Corners {
    fn from(radius: f32) -> Self {
        Self::all(radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_fields() {
        let c = Corners::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(c.top_left, 1.0);
        assert_eq!(c.top_right, 2.0);
        assert_eq!(c.bottom_right, 3.0);
        assert_eq!(c.bottom_left, 4.0);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Corners::default(), Corners::zero());
        assert!(Corners::default().is_zero());
    }

    #[test]
    fn all_uniform() {
        let c = Corners::all(8.0);
        assert_eq!(c.top_left, 8.0);
        assert_eq!(c.top_right, 8.0);
        assert_eq!(c.bottom_right, 8.0);
        assert_eq!(c.bottom_left, 8.0);
        assert!(c.is_uniform());
    }

    #[test]
    fn square_is_zero() {
        assert_eq!(Corners::square(), Corners::zero());
        assert!(Corners::square().is_square());
    }

    #[test]
    fn top_sets_both_top() {
        let c = Corners::zero().top(12.0);
        assert_eq!(c.top_left, 12.0);
        assert_eq!(c.top_right, 12.0);
        assert_eq!(c.bottom_left, 0.0);
        assert_eq!(c.bottom_right, 0.0);
    }

    #[test]
    fn bottom_sets_both_bottom() {
        let c = Corners::zero().bottom(12.0);
        assert_eq!(c.bottom_left, 12.0);
        assert_eq!(c.bottom_right, 12.0);
        assert_eq!(c.top_left, 0.0);
        assert_eq!(c.top_right, 0.0);
    }

    #[test]
    fn left_sets_both_left() {
        let c = Corners::zero().left(12.0);
        assert_eq!(c.top_left, 12.0);
        assert_eq!(c.bottom_left, 12.0);
        assert_eq!(c.top_right, 0.0);
        assert_eq!(c.bottom_right, 0.0);
    }

    #[test]
    fn right_sets_both_right() {
        let c = Corners::zero().right(12.0);
        assert_eq!(c.top_right, 12.0);
        assert_eq!(c.bottom_right, 12.0);
        assert_eq!(c.top_left, 0.0);
        assert_eq!(c.bottom_left, 0.0);
    }

    #[test]
    fn top_mut_chains() {
        let mut c = Corners::zero();
        c.top_mut(8.0).left_mut(4.0);
        assert_eq!(c.top_left, 4.0); // left_mut overwrites top_mut for top_left
        assert_eq!(c.top_right, 8.0);
        assert_eq!(c.bottom_left, 4.0);
        assert_eq!(c.bottom_right, 0.0);
    }

    #[test]
    fn bottom_mut() {
        let mut c = Corners::all(10.0);
        c.bottom_mut(5.0);
        assert_eq!(c.bottom_left, 5.0);
        assert_eq!(c.bottom_right, 5.0);
        assert_eq!(c.top_left, 10.0);
        assert_eq!(c.top_right, 10.0);
    }

    #[test]
    fn right_mut() {
        let mut c = Corners::zero();
        c.right_mut(7.0);
        assert_eq!(c.top_right, 7.0);
        assert_eq!(c.bottom_right, 7.0);
    }

    #[test]
    fn set_and_set_mut() {
        let c = Corners::new(1.0, 2.0, 3.0, 4.0).set(10.0);
        assert_eq!(c, Corners::all(10.0));

        let mut c2 = Corners::new(1.0, 2.0, 3.0, 4.0);
        c2.set_mut(10.0);
        assert_eq!(c2, Corners::all(10.0));
    }

    #[test]
    fn is_uniform_all_equal() {
        assert!(Corners::all(5.0).is_uniform());
        assert!(!Corners::new(1.0, 2.0, 3.0, 4.0).is_uniform());
        assert!(Corners::zero().is_uniform());
    }

    #[test]
    fn is_zero_checks_all() {
        assert!(Corners::zero().is_zero());
        assert!(!Corners::new(0.0, 0.0, 0.0, 1.0).is_zero());
    }

    #[test]
    fn is_square_checks_all_zero() {
        assert!(Corners::zero().is_square());
        assert!(!Corners::all(1.0).is_square());
    }

    #[test]
    fn array_roundtrip() {
        let c = Corners::new(1.0, 2.0, 3.0, 4.0);
        let arr = c.to_array();
        assert_eq!(arr, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(Corners::from_array(&arr), c);
    }

    #[test]
    fn from_f32() {
        let c: Corners = 8.0.into();
        assert_eq!(c, Corners::all(8.0));
    }

    #[test]
    fn immutable_chaining() {
        let c = Corners::zero().top(8.0).right(4.0);
        assert_eq!(c.top_left, 8.0);
        assert_eq!(c.top_right, 4.0); // right() overwrites top()'s top_right
        assert_eq!(c.bottom_right, 4.0);
        assert_eq!(c.bottom_left, 0.0);
    }
}
