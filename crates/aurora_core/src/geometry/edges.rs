use crate::geometry::size::Size;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            bottom,
            right,
            left,
        }
    }
    pub fn zero() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            right: 0.0,
            left: 0.0,
        }
    }
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            right: value,
            left: value,
        }
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Self::symmetric(x, y)
    }
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            right: horizontal,
            left: horizontal,
        }
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.horizontal(),
            height: self.vertical(),
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
    pub fn is_uniform(&self) -> bool {
        self.top == self.bottom && self.left == self.right
    }
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.bottom == 0.0 && self.left == 0.0 && self.right == 0.0
    }
}

impl From<f32> for Edges {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}
