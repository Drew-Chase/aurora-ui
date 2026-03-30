use crate::geometry::point::Point;
use std::path::PathBuf;

/// An event from an internal (widget-to-widget) drag gesture.
///
/// The platform dispatches these to the entire widget tree. Each widget
/// determines its role by inspecting `origin` (to find the drag source)
/// and `current` (to find the drop target).
#[derive(Debug, Clone)]
pub enum DragEvent {
    /// A drag gesture has started.
    ///
    /// Dispatched once when the cursor moves beyond the drag threshold
    /// after a mouse press. Widgets compare `origin` against their bounds
    /// to determine if they are the drag source.
    DragStart {
        /// Position where the mouse was originally pressed.
        origin: Point,
    },
    /// The drag gesture is in progress.
    ///
    /// Dispatched on every mouse move while a drag is active. Sources use
    /// `origin` to identify themselves; drop targets use `current` to track
    /// whether the cursor is over them.
    DragMove {
        /// Position where the drag originated (where mouse was pressed).
        origin: Point,
        /// Current cursor position.
        current: Point,
    },
    /// The drag gesture ended (mouse was released).
    ///
    /// Sources use `origin` to identify themselves and clean up state.
    /// Drop targets check whether `current` is within their bounds to
    /// trigger an accept/drop action.
    DragEnd {
        /// Position where the drag originated.
        origin: Point,
        /// Position where the mouse was released.
        current: Point,
    },
}

/// An event from an OS-level file drag-and-drop operation.
///
/// These events originate from the windowing system when the user drags
/// files from the OS file manager onto the application window.
#[derive(Debug, Clone)]
pub enum FileDropEvent {
    /// A file is being dragged over the window but not yet dropped.
    ///
    /// May be dispatched multiple times as the cursor moves.
    FileHovered {
        /// Path of the file being dragged over the window.
        path: PathBuf,
        /// Current cursor position within the window.
        position: Point,
    },
    /// A file was dropped onto the window.
    FileDropped {
        /// Path of the dropped file.
        path: PathBuf,
        /// Cursor position at the moment of the drop.
        position: Point,
    },
    /// The OS file drag was cancelled (user moved the file away).
    FileCancelled,
}
