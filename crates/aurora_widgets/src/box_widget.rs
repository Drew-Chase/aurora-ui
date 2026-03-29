use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// A basic rectangular container with background color, corner radii, and optional child.
///
/// This is the simplest visual widget — use it for colored rectangles, cards,
/// or as a styled wrapper around another widget.
pub struct BoxWidget {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background_color: aurora_core::color::Color,
    pub corners: Corners,
    pub padding: Edges,
    pub child: Option<Box<dyn Widget>>,
    child_rect: Rect,
}

impl BoxWidget {
    /// Creates a new box with default settings (transparent, no corners, no child).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a fixed width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the background fill color.
    pub fn background_color(mut self, color: aurora_core::color::Color) -> Self {
        self.background_color = color;
        self
    }

    /// Sets the corner radii for rounded corners.
    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    /// Sets the inner padding between the box edge and its child.
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    /// Sets a child widget to render inside the box.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Default for BoxWidget {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            background_color: aurora_core::color::Color::TRANSPARENT,
            corners: Corners::zero(),
            padding: Edges::zero(),
            child: None,
            child_rect: Rect::zero(),
        }
    }
}

impl Widget for BoxWidget {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let width = match self.width {
            Some(w) => w as f32,
            None => available.width,
        };

        // Layout child to determine content height
        let inner_w = (width - self.padding.horizontal()).max(0.0);
        let inner_h = match self.height {
            Some(h) => ((h as f32) - self.padding.vertical()).max(0.0),
            None => (available.height - self.padding.vertical()).max(0.0),
        };

        let child_size = if let Some(child) = &mut self.child {
            let content_area = Size::new(inner_w, inner_h);
            let cs = child.layout(content_area, ctx);
            self.child_rect = Rect::new(
                self.padding.left,
                self.padding.top,
                self.padding.left + cs.width.min(inner_w),
                self.padding.top + cs.height.min(inner_h),
            );
            cs
        } else {
            Size::default()
        };

        // Without explicit height, size to content instead of filling available space
        let height = match self.height {
            Some(h) => h as f32,
            None => child_size.height + self.padding.vertical(),
        };

        Size::new(width.min(available.width), height.min(available.height))
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rounded_rect(rect, self.corners, self.background_color);
        if let Some(child) = &self.child {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint(canvas, translated);
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let Some(child) = &self.child {
            let translated = self.child_rect.translate(&rect.origin());
            child.paint_overlay(canvas, translated);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if let Some(child) = &mut self.child {
            let translated = self.child_rect.translate(&rect.origin());
            child.event(event, translated)
        } else {
            EventResponse::default()
        }
    }
}
