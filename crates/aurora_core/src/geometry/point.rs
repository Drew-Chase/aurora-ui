#![doc = include_str!("../../../../.wiki/Point.md")]

use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A 2D point with `x` and `y` coordinates.
///
/// Represents a position in 2D space. Supports arithmetic via operator
/// overloading ([`Add`], [`Sub`], [`AddAssign`], [`SubAssign`]) with other
/// points, scalars, and tuples.
///
/// # Examples
///
/// ```
/// use aurora_core::geometry::point::Point;
///
/// let a = Point::new(10.0, 20.0);
/// let b = Point::new(5.0, 5.0);
/// let c = a + b; // Point { x: 15.0, y: 25.0 }
/// let d = a - 3.0; // Point { x: 7.0, y: 17.0 }
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    /// Creates a new point from the given coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;
    /// Adds two points component-wise.
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add<f32> for Point {
    type Output = Self;
    /// Adds a scalar to both components.
    fn add(self, other: f32) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl AddAssign<f32> for Point {
    /// Adds a scalar to both components in place.
    fn add_assign(&mut self, other: f32) {
        self.x += other;
        self.y += other;
    }
}

impl Add<(f32, f32)> for Point {
    type Output = Self;
    /// Adds a tuple `(dx, dy)` to the point.
    fn add(self, other: (f32, f32)) -> Self {
        Self {
            x: self.x + other.0,
            y: self.y + other.1,
        }
    }
}
impl AddAssign<(f32, f32)> for Point {
    /// Adds a tuple `(dx, dy)` to the point in place.
    fn add_assign(&mut self, other: (f32, f32)) {
        self.x += other.0;
        self.y += other.1;
    }
}

impl Sub for Point {
    type Output = Self;
    /// Subtracts two points component-wise.
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<f32> for Point {
    type Output = Self;
    /// Subtracts a scalar from both components.
    fn sub(self, other: f32) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl SubAssign<f32> for Point {
    /// Subtracts a scalar from both components in place.
    fn sub_assign(&mut self, other: f32) {
        self.x -= other;
        self.y -= other;
    }
}

impl Sub<(f32, f32)> for Point {
    type Output = Self;
    /// Subtracts a tuple `(dx, dy)` from the point.
    fn sub(self, other: (f32, f32)) -> Self {
        Self {
            x: self.x - other.0,
            y: self.y - other.1,
        }
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}
