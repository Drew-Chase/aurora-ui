use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add<f32> for Point {
    type Output = Self;
    fn add(self, other: f32) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl AddAssign<f32> for Point {
    fn add_assign(&mut self, other: f32) {
        self.x += other;
        self.y += other;
    }
}

impl Add<(f32, f32)> for Point {
    type Output = Self;
    fn add(self, other: (f32, f32)) -> Self {
        Self {
            x: self.x + other.0,
            y: self.y + other.1,
        }
    }
}
impl AddAssign<(f32, f32)> for Point {
    fn add_assign(&mut self, other: (f32, f32)) {
        self.x += other.0;
        self.y += other.1;
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<f32> for Point {
    type Output = Self;
    fn sub(self, other: f32) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl SubAssign<f32> for Point {
    fn sub_assign(&mut self, other: f32) {
        self.x -= other;
        self.y -= other;
    }
}

impl Sub<(f32, f32)> for Point {
    type Output = Self;
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
