#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
    pub fn all(radius: f32) -> Self {
        Corners {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
    pub fn zero() -> Self {
        Self::all(0.0)
    }
    pub fn top(&self, radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            ..*self
        }
    }

    pub fn top_mut(&mut self, radius: f32) -> &mut Self {
        self.top_left = radius;
        self.top_right = radius;
        self
    }

    pub fn bottom(&self, radius: f32) -> Self {
        Self {
            bottom_left: radius,
            bottom_right: radius,
            ..*self
        }
    }

    pub fn bottom_mut(&mut self, radius: f32) -> &mut Self {
        self.bottom_left = radius;
        self.bottom_right = radius;
        self
    }

    pub fn left(&self, radius: f32) -> Self {
        Self {
            top_left: radius,
            bottom_left: radius,
            ..*self
        }
    }

    pub fn left_mut(&mut self, radius: f32) -> &mut Self {
        self.top_left = radius;
        self.bottom_left = radius;
        self
    }

    pub fn right(&self, radius: f32) -> Self {
        Self {
            top_right: radius,
            bottom_right: radius,
            ..*self
        }
    }

    pub fn right_mut(&mut self, radius: f32) -> &mut Self {
        self.top_right = radius;
        self.bottom_right = radius;
        self
    }

    pub fn set(&self, radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn set_mut(&mut self, radius: f32) -> &mut Self {
        self.top_left = radius;
        self.top_right = radius;
        self.bottom_left = radius;
        self.bottom_right = radius;
        self
    }

    pub fn square() -> Self {
        Self::all(0.0)
    }
    pub fn is_square(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_left == 0.0
            && self.bottom_right == 0.0
    }
    pub fn to_array(&self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }
    pub fn from_array(array: &[f32; 4]) -> Self {
        Self {
            top_left: array[0],
            top_right: array[1],
            bottom_right: array[2],
            bottom_left: array[3],
        }
    }
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0 && self.top_right == 0.0 && self.bottom_left == 0.0 && self.bottom_right == 0.0
    }
}

impl From<f32> for Corners {
    fn from(radius: f32) -> Self {
        Self::all(radius)
    }
}