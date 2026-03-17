/// The visual appearance of the mouse cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorIcon {
    /// Platform default arrow cursor.
    Default,
    /// Hand/pointer cursor, typically used for clickable elements.
    Pointer,
    /// I-beam cursor for text selection.
    Text,
    /// Open hand cursor for draggable elements.
    Grab,
    /// Closed hand cursor while actively dragging.
    Grabbing,
    /// Circle-slash cursor indicating a disallowed action.
    NotAllowed,
}
