use crate::geometry::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseButton{
	Left,
	Middle,
	Right,
	Back,
	Forward,
	ScrollUp,
	ScrollDown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseState{
	Pressed,
	Released,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent{
	MouseClickEvent(MouseClickEvent),
	MouseMoveEvent(Point),
	MouseScrollEvent(bool)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseClickEvent{
	pub button: MouseButton,
	pub state: MouseState,
	pub position: Point,
}
