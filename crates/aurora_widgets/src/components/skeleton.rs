use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A loading placeholder with a shimmer effect.
///
/// # Example
/// ```ignore
/// Skeleton::new()
///     .width(200)
///     .height(20)
///     .corners(Corners::all(4.0))
/// ```
pub struct Skeleton {
    width: Option<f32>,
    height: Option<f32>,
    color: Color,
    corners: Corners,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: None,
            height: Some(20.0),
            color: colors::border(),
            corners: Corners::all(4.0),
        }
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    /// Creates a circle skeleton (avatar placeholder).
    pub fn circle(size: f32) -> Self {
        Self {
            width: Some(size),
            height: Some(size),
            color: colors::muted(),
            corners: Corners::all(size / 2.0),
        }
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Skeleton {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        Size::new(
            self.width.unwrap_or(available.width),
            self.height.unwrap_or(20.0),
        )
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rounded_rect(rect, self.corners, self.color);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
