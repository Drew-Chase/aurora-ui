use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A loading spinner indicator.
///
/// Draws a circular track with a partial arc to indicate loading.
/// Without animation feature, displays a static partial ring.
///
/// # Example
/// ```ignore
/// Spinner::new()
///     .size(24.0)
///     .color(Color::new(59, 130, 246, 255))
/// ```
pub struct Spinner {
    diameter: f32,
    track_color: Color,
    color: Color,
    thickness: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            diameter: 24.0,
            track_color: colors::MUTED,
            color: colors::PRIMARY,
            thickness: 3.0,
        }
    }

    pub fn size(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn layout(&mut self, _available: Size, _ctx: &mut LayoutCtx) -> Size {
        Size::new(self.diameter, self.diameter)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let cx = rect.x1 + rect.width() / 2.0;
        let cy = rect.y1 + rect.height() / 2.0;
        let r = (self.diameter / 2.0) - self.thickness / 2.0;

        // Draw a circle track using stroke
        let track_rect = Rect::new(
            cx - r - self.thickness / 2.0,
            cy - r - self.thickness / 2.0,
            cx + r + self.thickness / 2.0,
            cy + r + self.thickness / 2.0,
        );
        let corners = aurora_core::geometry::corners::Corners::all(r + self.thickness / 2.0);
        canvas.stroke_rounded_rect(track_rect, corners, self.thickness as u32, self.track_color);

        // Draw a partial arc (top-right quadrant) as a simple indicator
        // Since we don't have arc drawing, draw a small filled circle at the top
        let indicator_size = self.thickness * 2.0;
        let indicator_rect = Rect::new(
            cx - indicator_size / 2.0,
            rect.y1,
            cx + indicator_size / 2.0,
            rect.y1 + indicator_size,
        );
        let ind_corners = aurora_core::geometry::corners::Corners::all(indicator_size / 2.0);
        canvas.fill_rounded_rect(indicator_rect, ind_corners, self.color);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
