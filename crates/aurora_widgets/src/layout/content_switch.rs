use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// A container that displays exactly one child at a time based on the
/// selected index. Use this to build multi-page interfaces where only
/// one page is visible.
///
/// # Example
/// ```ignore
/// ContentSwitch::new()
///     .selected(current_page)
///     .item(page_one_widget)
///     .item(page_two_widget)
///     .item(page_three_widget)
/// ```
pub struct ContentSwitch {
    items: Vec<Box<dyn Widget>>,
    selected: usize,
}

impl ContentSwitch {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
        }
    }

    /// Sets which item index to display.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Appends a child widget as the next item.
    pub fn item(mut self, widget: impl Widget + 'static) -> Self {
        self.items.push(Box::new(widget));
        self
    }
}

impl Default for ContentSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ContentSwitch {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if let Some(child) = self.items.get_mut(self.selected) {
            child.layout(available, ctx)
        } else {
            Size::default()
        }
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = self.items.get(self.selected) {
            child.paint(canvas, rect);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        if self.selected < self.items.len() {
            &self.items[self.selected..=self.selected]
        } else {
            &[]
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if let Some(child) = self.items.get_mut(self.selected) {
            child.event(event, rect)
        } else {
            EventResponse::default()
        }
    }
}
