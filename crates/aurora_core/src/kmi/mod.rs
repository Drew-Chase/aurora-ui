/// Mouse cursor icon variants.
pub mod cursor_icon;
/// Keyboard input events.
pub mod keyboard;
/// Mouse input events and button/state types.
pub mod mouse;

use keyboard::KeyboardEvent;
use mouse::MouseEvent;

/// A unified input event dispatched to widgets.
#[derive(Debug, Clone)]
pub enum WidgetEvent {
    /// A mouse input event.
    Mouse(MouseEvent),
    /// A keyboard input event.
    Keyboard(KeyboardEvent),
    /// Request that the widget with this ID takes focus.
    /// The bool controls whether to select all text (true for tab navigation,
    /// false for focus restoration after Composite rebuilds).
    Focus(u64, bool),
    /// Request that the widget with this ID loses focus.
    Blur(u64),
}
