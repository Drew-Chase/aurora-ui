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
