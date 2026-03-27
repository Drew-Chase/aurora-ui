use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// The result of dispatching an event to a widget.
#[derive(Default)]
pub struct EventResponse {
    /// Whether this widget (or a descendant) consumed the event.
    pub handled: bool,
    /// Cursor icon to show while hovering this widget, if any.
    pub cursor: Option<CursorIcon>,
    /// Request focus for the widget with this ID, if any.
    pub request_focus: Option<u64>,
    /// Request that focus move to the next focusable widget (Tab navigation).
    pub focus_next: bool,
    /// Request that focus move to the previous focusable widget (Shift+Tab).
    pub focus_prev: bool,
}

/// A UI element that can be laid out and painted.
///
/// Implement this trait to create custom widgets. The framework calls
/// [`layout`](Widget::layout) first to determine sizes, then [`paint`](Widget::paint)
/// to draw the widget into a [`Canvas`].
pub trait Widget {
    /// Computes the widget's size given the available space and a layout context.
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size;
    /// Draws the widget into the canvas at the given rectangle.
    fn paint(&self, canvas: &mut Canvas, rect: Rect);
    /// Draws overlay content (dropdowns, popups) that must appear above all siblings.
    /// Called after the entire widget tree has been painted via [`paint`].
    /// Containers must override this to forward to children.
    fn paint_overlay(&self, _canvas: &mut Canvas, _rect: Rect) {}
    /// Returns the widget's child widgets.
    fn children(&self) -> &[Box<dyn Widget>];
    /// Handles a widget event within the given bounds. Returns an [`EventResponse`].
    ///
    /// The default implementation ignores the event.
    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
    /// Returns this widget's tab index for keyboard navigation, if focusable.
    fn tab_index(&self) -> Option<u32> {
        None
    }
    /// Returns this widget's unique ID, if it has one.
    fn widget_id(&self) -> Option<u64> {
        None
    }
    /// Returns `true` if this widget (or any descendant) requires continuous
    /// animation frames. The default walks [`children`](Widget::children).
    fn needs_animation(&self) -> bool {
        self.children().iter().any(|c| c.needs_animation())
    }

    /// Returns accessibility information for this widget.
    ///
    /// The default returns a transparent node, meaning the widget has no
    /// semantic role and its children are promoted into the parent's child
    /// list in the accessibility tree. Override this to declare a semantic
    /// role (button, checkbox, text input, etc.).
    ///
    /// Requires the `a11y` feature.
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::transparent()
    }
}

/// Context passed to [`Widget::layout`] during the layout phase.
///
/// When the `text` feature is enabled, this carries a mutable reference to
/// the [`FontManager`](aurora_text::font_manager::FontManager) and the global
/// [`FontOptions`](aurora_text::font_options::FontOptions) so widgets can shape
/// text with the application's default font settings.
#[cfg(feature = "text")]
pub struct LayoutCtx<'a> {
    pub font_manager: &'a mut aurora_text::font_manager::FontManager,
    pub font_options: &'a aurora_text::font_options::FontOptions,
}

/// Context passed to [`Widget::layout`] during the layout phase.
///
/// This is the text-free variant; no font manager is available.
#[cfg(not(feature = "text"))]
pub struct LayoutCtx;
