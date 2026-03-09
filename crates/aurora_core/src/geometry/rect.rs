use crate::geometry::edges::Edges;
use crate::geometry::point::Point;
use crate::geometry::size::Size;
use std::ops::{Add, AddAssign};

/// An axis-aligned rectangle defined by two corner points `(x1, y1)` and `(x2, y2)`.
///
/// `(x1, y1)` is the top-left corner and `(x2, y2)` is the bottom-right corner.
/// A valid rect satisfies `x1 <= x2` and `y1 <= y2`.
///
/// Most mutation methods come in two variants:
/// - **Immutable** (e.g. [`inset`](Self::inset)) — returns a new `Rect` with the change applied.
/// - **Mutable** (e.g. [`inset_mut`](Self::inset_mut)) — modifies in place and returns `&mut Self` for chaining.
///
/// # Examples
///
/// ```
/// use aurora_core::geometry::rect::Rect;
/// use aurora_core::geometry::point::Point;
/// use aurora_core::geometry::size::Size;
///
/// let rect = Rect::from_origin_size(Point::new(10.0, 20.0), Size::new(100.0, 50.0));
/// assert_eq!(rect.width(), 100.0);
/// assert_eq!(rect.height(), 50.0);
/// assert!(rect.contains(&Point::new(50.0, 40.0)));
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Rect {
    /// Creates a rectangle from explicit corner coordinates.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Creates a rectangle from an origin point and a size.
    pub fn from_origin_size(point: Point, size: Size) -> Self {
        Self::new(
            point.x,
            point.y,
            point.x + size.width,
            point.y + size.height,
        )
    }

    /// Creates a rectangle at the origin `(0, 0)` with the given size.
    pub fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }

    /// Returns a zero-sized rectangle at the origin.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Returns a new rectangle shrunk inward by the given edges.
    ///
    /// The resulting rectangle is clamped so that `x2 >= x1` and `y2 >= y1`.
    pub fn inset(&self, edges: &Edges) -> Self {
        let x1 = self.x1 + edges.left;
        let y1 = self.y1 + edges.top;
        Self {
            x1,
            y1,
            x2: (self.x2 - edges.right).max(x1),
            y2: (self.y2 - edges.bottom).max(y1),
        }
    }

    /// Shrinks this rectangle inward by the given edges in place.
    ///
    /// The result is clamped so that `x2 >= x1` and `y2 >= y1`.
    pub fn inset_mut(&mut self, edges: &Edges) -> &mut Self {
        self.x1 += edges.left;
        self.y1 += edges.top;
        self.x2 = (self.x2 - edges.right).max(self.x1);
        self.y2 = (self.y2 - edges.bottom).max(self.y1);
        self
    }

    /// Returns a new rectangle expanded outward by the given edges.
    pub fn outset(&self, edges: &Edges) -> Self {
        Self {
            x1: self.x1 - edges.left,
            y1: self.y1 - edges.top,
            x2: self.x2 + edges.right,
            y2: self.y2 + edges.bottom,
        }
    }

    /// Expands this rectangle outward by the given edges in place.
    pub fn outset_mut(&mut self, edges: &Edges) -> &mut Self {
        self.x1 -= edges.left;
        self.y1 -= edges.top;
        self.x2 += edges.right;
        self.y2 += edges.bottom;
        self
    }

    /// Returns `true` if all four coordinates are equal.
    pub fn is_uniform(&self) -> bool {
        [self.x1, self.y1, self.x2, self.y2]
            .windows(2)
            .all(|w| w[0] == w[1])
    }

    /// Returns a new rectangle repositioned to the given origin, preserving its size.
    pub fn set_origin(&self, point: &Point) -> Self {
        let width = self.width();
        let height = self.height();
        Self {
            x1: point.x,
            y1: point.y,
            x2: point.x + width,
            y2: point.y + height,
        }
    }

    /// Repositions this rectangle to the given origin in place, preserving its size.
    pub fn set_origin_mut(&mut self, point: &Point) -> &mut Self {
        let width = self.width();
        let height = self.height();
        self.x1 = point.x;
        self.y1 = point.y;
        self.x2 = point.x + width;
        self.y2 = point.y + height;
        self
    }

    /// Returns a new rectangle offset by the given point.
    pub fn translate(&self, point: &Point) -> Self {
        Self {
            x1: self.x1 + point.x,
            y1: self.y1 + point.y,
            x2: self.x2 + point.x,
            y2: self.y2 + point.y,
        }
    }

    /// Offsets this rectangle by the given point in place.
    pub fn translate_mut(&mut self, point: &Point) -> &mut Self {
        self.x1 += point.x;
        self.y1 += point.y;
        self.x2 += point.x;
        self.y2 += point.y;
        self
    }

    /// Returns `true` if the width equals the height.
    pub fn is_square(&self) -> bool {
        self.width() == self.height()
    }

    /// Returns `true` if all coordinates are zero.
    pub fn is_zero(&self) -> bool {
        self.x1 == 0.0 && self.y1 == 0.0 && self.x2 == 0.0 && self.y2 == 0.0
    }

    /// Returns the width (`x2 - x1`).
    pub fn width(&self) -> f32 {
        self.x2 - self.x1
    }

    /// Returns the height (`y2 - y1`).
    pub fn height(&self) -> f32 {
        self.y2 - self.y1
    }

    /// Returns the size of the rectangle as a [`Size`].
    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    /// Returns `true` if this rectangle overlaps with `other`.
    pub fn intersects(&self, other: &Self) -> bool {
        self.x1 < other.x2 && self.x2 > other.x1 && self.y1 < other.y2 && self.y2 > other.y1
    }

    /// Returns the overlapping region between two rectangles, or `None` if they don't overlap.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        Some(Self::new(
            self.x1.max(other.x1),
            self.y1.max(other.y1),
            self.x2.min(other.x2),
            self.y2.min(other.y2),
        ))
    }

    /// Returns `true` if the given point lies within this rectangle (inclusive).
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x1 && point.x <= self.x2 && point.y >= self.y1 && point.y <= self.y2
    }

    /// Returns the smallest rectangle that contains both `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        Self::new(
            self.x1.min(other.x1),
            self.y1.min(other.y1),
            self.x2.max(other.x2),
            self.y2.max(other.y2),
        )
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Point {
        Point {
            x: (self.x1 + self.x2) / 2.0,
            y: (self.y1 + self.y2) / 2.0,
        }
    }

    /// Returns `true` if `x1 <= x2` and `y1 <= y2`.
    pub fn is_valid(&self) -> bool {
        self.x1 <= self.x2 && self.y1 <= self.y2
    }
}

impl Add<Size> for Rect {
    type Output = Self;
    /// Extends the rectangle's bottom-right corner by the given size.
    fn add(self, other: Size) -> Self {
        Self::new(
            self.x1,
            self.y1,
            self.x2 + other.width,
            self.y2 + other.height,
        )
    }
}

impl AddAssign<Size> for Rect {
    /// Extends the rectangle's bottom-right corner by the given size in place.
    fn add_assign(&mut self, other: Size) {
        self.x2 += other.width;
        self.y2 += other.height;
    }
}

impl From<(f32, f32, f32, f32)> for Rect {
    fn from((x1, y1, x2, y2): (f32, f32, f32, f32)) -> Self {
        Self::new(x1, y1, x2, y2)
    }
}
