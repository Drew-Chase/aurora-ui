#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
	pub fn new(width: f32, height: f32)->Self{
		Self{
			width,
			height,
		}
	}
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
	pub fn is_zero(&self)->bool{
		self.width == 0.0 && self.height == 0.0
	}

	pub fn contains(&self, other: &Self) -> bool {
		self.width >= other.width && self.height >= other.height
	}

	pub fn area(&self)->f32{
		self.width * self.height
	}

	pub fn is_valid(&self)->bool{
		self.width > 0.0 && self.height > 0.0
	}

	pub fn is_square(&self)->bool{
		self.width == self.height
	}

}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self { width, height }
    }
}
