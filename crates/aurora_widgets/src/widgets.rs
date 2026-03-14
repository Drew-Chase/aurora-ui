use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::MouseEvent;
use aurora_render::canvas::Canvas;
#[derive(Default)]
pub struct EventResponse {
    pub handled: bool,
    pub cursor: Option<CursorIcon>,
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
    /// Returns the widget's child widgets.
    fn children(&self) -> &[Box<dyn Widget>];
    fn event(&mut self, _event: &MouseEvent, _rect: Rect)->EventResponse{
        EventResponse::default()
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
