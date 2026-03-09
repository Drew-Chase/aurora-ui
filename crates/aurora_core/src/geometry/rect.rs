use crate::geometry::edges::Edges;
use crate::geometry::point::Point;
use crate::geometry::size::Size;
use std::ops::{Add, AddAssign};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Rect {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn from_origin_size(point: Point, size: Size) -> Self {
        Self::new(
            point.x,
            point.y,
            point.x + size.width,
            point.y + size.height,
        )
    }

    pub fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
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

    pub fn inset_mut(&mut self, edges: &Edges) -> &mut Self {
        self.x1 = self.x1 + edges.left;
        self.y1 = self.y1 + edges.top;
        self.x2 = (self.x2 - edges.right).max(self.x1);
        self.y2 = (self.y2 - edges.bottom).max(self.y1);
        self
    }

    pub fn outset(&self, edges: &Edges) -> Self {
        Self {
            x1: self.x1 - edges.left,
            y1: self.y1 - edges.top,
            x2: self.x2 + edges.right,
            y2: self.y2 + edges.bottom,
        }
    }

    pub fn outset_mut(&mut self, edges: &Edges) -> &mut Self {
        self.x1 -= edges.left;
        self.y1 -= edges.top;
        self.x2 += edges.right;
        self.y2 += edges.bottom;
        self
    }
    pub fn is_uniform(&self) -> bool {
        [self.x1, self.y1, self.x2, self.y2]
            .windows(2)
            .all(|w| w[0] == w[1])
    }

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

    pub fn set_origin_mut(&mut self, point: &Point) -> &mut Self {
        let width = self.width();
        let height = self.height();
        self.x1 = point.x;
        self.y1 = point.y;
        self.x2 = point.x + width;
        self.y2 = point.y + height;
        self
    }

    pub fn translate(&self, point: &Point) -> Self {
        Self {
            x1: self.x1 + point.x,
            y1: self.y1 + point.y,
            x2: self.x2 + point.x,
            y2: self.y2 + point.y,
        }
    }

    pub fn translate_mut(&mut self, point: &Point) -> &mut Self {
        self.x1 += point.x;
        self.y1 += point.y;
        self.x2 += point.x;
        self.y2 += point.y;
        self
    }

    pub fn is_square(&self) -> bool {
        self.width() == self.height()
    }
    pub fn is_zero(&self) -> bool {
        self.x1 == 0.0 && self.y1 == 0.0 && self.x2 == 0.0 && self.y2 == 0.0
    }

    pub fn width(&self) -> f32 {
        self.x2 - self.x1
    }
    pub fn height(&self) -> f32 {
        self.y2 - self.y1
    }
    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
    pub fn intersects(&self, other: &Self) -> bool {
        self.x1 < other.x2 && self.x2 > other.x1 && self.y1 < other.y2 && self.y2 > other.y1
    }
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
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x1 && point.x <= self.x2 && point.y >= self.y1 && point.y <= self.y2
    }
    pub fn union(&self, other: &Self) -> Self {
        Self::new(
            self.x1.min(other.x1),
            self.y1.min(other.y1),
            self.x2.max(other.x2),
            self.y2.max(other.y2),
        )
    }
    pub fn center(&self) -> Point {
        Point {
            x: (self.x1 + self.x2) / 2.0,
            y: (self.y1 + self.y2) / 2.0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.x1 <= self.x2 && self.y1 <= self.y2
    }
}

impl Add<Size> for Rect {
    type Output = Self;
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
