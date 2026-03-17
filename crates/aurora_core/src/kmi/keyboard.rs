/// A keyboard input event.
pub enum KeyboardEvent {
    /// A key was pressed. The `u16` is the virtual key code.
    KeyDown(u16),
    /// A key was released. The `u16` is the virtual key code.
    KeyUp(u16),
}
