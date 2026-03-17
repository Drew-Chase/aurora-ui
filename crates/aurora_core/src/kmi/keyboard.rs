/// A keyboard input event.
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// A key was pressed.
    KeyPressed {
        /// The logical key that was pressed.
        key: Key,
        /// Active modifier keys.
        modifiers: Modifiers,
    },
    /// A key was released.
    KeyReleased {
        /// The logical key that was released.
        key: Key,
        /// Active modifier keys.
        modifiers: Modifiers,
    },
    /// A character was typed (after OS processing of dead keys, compose, layout).
    CharTyped(char),
}

/// A logical key identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A printable character.
    Character(char),
    /// Backspace (delete backward).
    Backspace,
    /// Delete (delete forward).
    Delete,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Enter / Return.
    Enter,
    /// Tab key.
    Tab,
    /// Escape key.
    Escape,
    /// A key not covered by other variants.
    Other,
}

/// Modifier key state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift is held.
    pub shift: bool,
    /// Ctrl (or Cmd on macOS) is held.
    pub ctrl: bool,
    /// Alt (or Option on macOS) is held.
    pub alt: bool,
}
