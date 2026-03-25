use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;

/// A type that can be smoothly interpolated between two values.
///
/// `t` is always in the range 0.0..=1.0 (already eased by the animation system).
/// Implementations linearly interpolate between `self` (t=0) and `other` (t=1).
pub trait Animatable: Copy {
    /// Linearly interpolates between `self` and `other` by `t`.
    ///
    /// - `t = 0.0` returns `self`
    /// - `t = 1.0` returns `other`
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Animatable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Animatable for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Color::lerp(self, other, t)
    }
}

impl Animatable for Point {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Point::new(
            Animatable::lerp(&self.x, &other.x, t),
            Animatable::lerp(&self.y, &other.y, t),
        )
    }
}

impl Animatable for Size {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Size::new(
            Animatable::lerp(&self.width, &other.width, t),
            Animatable::lerp(&self.height, &other.height, t),
        )
    }
}

impl Animatable for Rect {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Rect::new(
            Animatable::lerp(&self.x1, &other.x1, t),
            Animatable::lerp(&self.y1, &other.y1, t),
            Animatable::lerp(&self.x2, &other.x2, t),
            Animatable::lerp(&self.y2, &other.y2, t),
        )
    }
}

impl Animatable for Edges {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Edges::new(
            Animatable::lerp(&self.top, &other.top, t),
            Animatable::lerp(&self.right, &other.right, t),
            Animatable::lerp(&self.bottom, &other.bottom, t),
            Animatable::lerp(&self.left, &other.left, t),
        )
    }
}

impl Animatable for Corners {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Corners::new(
            Animatable::lerp(&self.top_left, &other.top_left, t),
            Animatable::lerp(&self.top_right, &other.top_right, t),
            Animatable::lerp(&self.bottom_right, &other.bottom_right, t),
            Animatable::lerp(&self.bottom_left, &other.bottom_left, t),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_lerp() {
        assert_eq!(Animatable::lerp(&0.0f32, &100.0, 0.0), 0.0);
        assert_eq!(Animatable::lerp(&0.0f32, &100.0, 0.5), 50.0);
        assert_eq!(Animatable::lerp(&0.0f32, &100.0, 1.0), 100.0);
    }

    #[test]
    fn f32_lerp_overshoot() {
        // Elastic/back easings can produce t > 1.0 — f32 lerp must allow this
        assert!((Animatable::lerp(&0.0f32, &100.0, 1.2) - 120.0).abs() < 0.001);
        assert!((Animatable::lerp(&0.0f32, &100.0, -0.1) - -10.0).abs() < 0.001);
    }

    #[test]
    fn color_lerp() {
        let a = Color::new(0, 0, 0, 255);
        let b = Color::new(255, 255, 255, 255);
        let mid = Animatable::lerp(&a, &b, 0.5);
        assert_eq!(mid.red, 128);
        assert_eq!(mid.green, 128);
        assert_eq!(mid.blue, 128);
    }

    #[test]
    fn point_lerp() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(100.0, 200.0);
        let mid = Animatable::lerp(&a, &b, 0.5);
        assert_eq!(mid.x, 50.0);
        assert_eq!(mid.y, 100.0);
    }

    #[test]
    fn size_lerp() {
        let a = Size::new(10.0, 20.0);
        let b = Size::new(30.0, 40.0);
        let mid = Animatable::lerp(&a, &b, 0.5);
        assert_eq!(mid.width, 20.0);
        assert_eq!(mid.height, 30.0);
    }

    #[test]
    fn rect_lerp() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 200.0, 200.0);
        let mid = Animatable::lerp(&a, &b, 0.5);
        assert_eq!(mid.x1, 25.0);
        assert_eq!(mid.y1, 25.0);
        assert_eq!(mid.x2, 150.0);
        assert_eq!(mid.y2, 150.0);
    }

    #[test]
    fn edges_lerp() {
        let a = Edges::new(0.0, 0.0, 0.0, 0.0);
        let b = Edges::new(10.0, 20.0, 30.0, 40.0);
        let mid = Animatable::lerp(&a, &b, 0.5);
        assert_eq!(mid.top, 5.0);
        assert_eq!(mid.right, 10.0);
        assert_eq!(mid.bottom, 15.0);
        assert_eq!(mid.left, 20.0);
    }

    #[test]
    fn corners_lerp() {
        let a = Corners::new(0.0, 0.0, 0.0, 0.0);
        let b = Corners::new(10.0, 20.0, 30.0, 40.0);
        let mid = Animatable::lerp(&a, &b, 0.5);
        assert_eq!(mid.top_left, 5.0);
        assert_eq!(mid.top_right, 10.0);
        assert_eq!(mid.bottom_right, 15.0);
        assert_eq!(mid.bottom_left, 20.0);
    }
}
