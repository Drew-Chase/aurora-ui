#![doc = include_str!("../../../../.wiki/Size.md")]

/// A 2D size with `width` and `height` dimensions.
///
/// Represents the dimensions of a rectangular area. Commonly used alongside
/// [`Rect`](super::rect::Rect) and [`Point`](super::point::Point) to describe
/// layout geometry.
///
/// # Examples
///
/// ```
/// use aurora_core::geometry::size::Size;
///
/// let size = Size::new(800.0, 600.0);
/// assert_eq!(size.area(), 480_000.0);
/// assert!(!size.is_square());
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Creates a new size from the given dimensions.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Returns a zero-sized `Size` (0 x 0).
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    /// Returns `true` if both dimensions are zero.
    pub fn is_zero(&self) -> bool {
        self.width == 0.0 && self.height == 0.0
    }

    /// Returns `true` if this size can fully contain `other`.
    pub fn contains(&self, other: &Self) -> bool {
        self.width >= other.width && self.height >= other.height
    }

    /// Returns the area (`width * height`).
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Returns `true` if both dimensions are positive (greater than zero).
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }

    /// Returns `true` if `width` equals `height`.
    pub fn is_square(&self) -> bool {
        self.width == self.height
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self { width, height }
    }
}
impl From<(i32, i32)> for Size {
    fn from((width, height): (i32, i32)) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_dimensions() {
        let s = Size::new(100.0, 200.0);
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 200.0);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Size::default(), Size::new(0.0, 0.0));
    }

    #[test]
    fn zero_constructor() {
        let s = Size::zero();
        assert_eq!(s.width, 0.0);
        assert_eq!(s.height, 0.0);
        assert!(s.is_zero());
    }

    #[test]
    fn is_zero_false_for_nonzero() {
        assert!(!Size::new(1.0, 0.0).is_zero());
        assert!(!Size::new(0.0, 1.0).is_zero());
    }

    #[test]
    fn is_valid_positive_dimensions() {
        assert!(Size::new(1.0, 1.0).is_valid());
        assert!(!Size::new(0.0, 1.0).is_valid());
        assert!(!Size::new(1.0, 0.0).is_valid());
        assert!(!Size::new(-1.0, 1.0).is_valid());
    }

    #[test]
    fn is_square() {
        assert!(Size::new(50.0, 50.0).is_square());
        assert!(!Size::new(50.0, 60.0).is_square());
        assert!(Size::zero().is_square());
    }

    #[test]
    fn area() {
        assert_eq!(Size::new(10.0, 20.0).area(), 200.0);
        assert_eq!(Size::zero().area(), 0.0);
    }

    #[test]
    fn contains_larger() {
        let big = Size::new(100.0, 100.0);
        let small = Size::new(50.0, 50.0);
        assert!(big.contains(&small));
        assert!(!small.contains(&big));
    }

    #[test]
    fn contains_equal() {
        let s = Size::new(100.0, 100.0);
        assert!(s.contains(&s));
    }

    #[test]
    fn from_f32_tuple() {
        let s: Size = (800.0, 600.0).into();
        assert_eq!(s, Size::new(800.0, 600.0));
    }

    #[test]
    fn from_i32_tuple() {
        let s: Size = (800, 600).into();
        assert_eq!(s, Size::new(800.0, 600.0));
    }

    #[test]
    fn contains_partial_overlap() {
        let a = Size::new(100.0, 50.0);
        let b = Size::new(50.0, 100.0);
        assert!(!a.contains(&b));
        assert!(!b.contains(&a));
    }
}
