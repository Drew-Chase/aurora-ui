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
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    /// Creates corners from individual radii (top-left, top-right, bottom-right, bottom-left).
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Returns corners where all four radii share the same value.
    pub fn all(radius: f32) -> Self {
        Corners {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Returns corners with all radii set to zero.
    pub fn zero() -> Self {
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
