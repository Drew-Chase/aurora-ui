use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A card container with optional header, content, and footer sections.
///
/// # Example
/// ```ignore
/// Card::new()
///     .child(Label::new("Card Title"))
///     .child(Text::new("Card content here"))
/// ```
pub struct Card {
    children: Vec<Box<dyn Widget>>,
    background_color: Color,
    border_color: Color,
    border_width: f32,
    corners: Corners,
    padding: Edges,
    spacing: f32,
    width: Option<f32>,
    height: Option<f32>,
    child_sizes: Vec<Size>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            background_color: colors::card(),
            border_color: colors::border(),
            border_width: 1.0,
            corners: Corners::all(8.0),
            padding: Edges::new(24.0, 24.0, 24.0, 24.0),
            spacing: 8.0,
            width: None,
            height: None,
            child_sizes: Vec::new(),
        }
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Card {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let inner_w = self.width.unwrap_or(available.width) - self.padding.left - self.padding.right;
        let child_available = Size::new(inner_w.max(0.0), f32::MAX);

        self.child_sizes.clear();
        let mut total_h = 0.0;
        for (i, child) in self.children.iter_mut().enumerate() {
            let cs = child.layout(child_available, ctx);
            self.child_sizes.push(cs);
            total_h += cs.height;
            if i > 0 {
                total_h += self.spacing;
            }
        }

        let w = self.width.unwrap_or(available.width);
        let h = self.height.unwrap_or(total_h + self.padding.top + self.padding.bottom);
        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background
        canvas.fill_rounded_rect(rect, self.corners, self.background_color);
        // Border
        if self.border_width > 0.0 {
            canvas.stroke_rounded_rect(rect, self.corners, self.border_width as u32, self.border_color);
        }
        // Children
        let mut y = rect.y1 + self.padding.top;
        for (i, child) in self.children.iter().enumerate() {
            let cs = self.child_sizes.get(i).copied().unwrap_or_default();
            let child_rect = Rect::new(
                rect.x1 + self.padding.left,
                y,
                rect.x1 + self.padding.left + cs.width,
                y + cs.height,
            );
            child.paint(canvas, child_rect);
            y += cs.height + self.spacing;
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let mut y = rect.y1 + self.padding.top;
        for (i, child) in self.children.iter_mut().enumerate() {
            let cs = self.child_sizes.get(i).copied().unwrap_or_default();
            let child_rect = Rect::new(
                rect.x1 + self.padding.left,
                y,
                rect.x1 + self.padding.left + cs.width,
                y + cs.height,
            );
            let resp = child.event(event, child_rect);
            if resp.handled {
                return resp;
            }
            y += cs.height + self.spacing;
        }
        EventResponse::default()
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)    }
}
