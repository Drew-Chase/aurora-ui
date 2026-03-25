use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// A container that enforces a specific aspect ratio on its child.
///
/// # Example
/// ```ignore
/// AspectRatio::new(16.0 / 9.0)
///     .child(Image::from_data(data))
/// ```
pub struct AspectRatio {
    ratio: f32,
    child: Option<Box<dyn Widget>>,
    width: Option<f32>,
    child_size: Size,
}

impl AspectRatio {
    /// Creates an aspect ratio container. `ratio` is width/height (e.g. 16.0/9.0).
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.max(0.001),
            child: None,
            width: None,
            child_size: Size::default(),
        }
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// 16:9 aspect ratio.
    pub fn widescreen() -> Self {
        Self::new(16.0 / 9.0)
    }

    /// 4:3 aspect ratio.
    pub fn standard() -> Self {
        Self::new(4.0 / 3.0)
    }

    /// 1:1 aspect ratio.
    pub fn square() -> Self {
        Self::new(1.0)
    }
}

impl Widget for AspectRatio {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let h = w / self.ratio;
        let size = Size::new(w, h);

        if let Some(ref mut child) = self.child {
            self.child_size = child.layout(size, ctx);
        }

        size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.push_clip(rect);
        if let Some(ref child) = self.child {
            child.paint(canvas, rect);
        }
        canvas.pop_clip();
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if let Some(ref mut child) = self.child {
            child.event(event, rect)
        } else {
            EventResponse::default()
        }
    }
}
