use crate::geometry::point::Point;

/// A mouse button identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseButton {
    /// Primary (left) button.
    Left,
    /// Middle (wheel) button.
    Middle,
    /// Secondary (right) button.
    Right,
    /// Back navigation button.
    Back,
    /// Forward navigation button.
    Forward,
    /// Scroll wheel up.
    ScrollUp,
    /// Scroll wheel down.
    ScrollDown,
}

/// Whether a mouse button is pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseState {
    /// The button was pressed down.
    Pressed,
    /// The button was released.
    Released,
}

/// A mouse input event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent {
    /// A mouse button was clicked (pressed or released).
    MouseClickEvent(MouseClickEvent),
    /// The mouse cursor moved to a new position.
    MouseMoveEvent(Point),
    /// The scroll wheel was used. Positive = scroll up, negative = scroll down.
    MouseScrollEvent(f32),
}

/// Details of a mouse click (press or release).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseClickEvent {
    /// Which button was involved.
    pub button: MouseButton,
    /// Whether the button was pressed or released.
    pub state: MouseState,
    /// Cursor position at the time of the event.
    pub position: Point,
}
