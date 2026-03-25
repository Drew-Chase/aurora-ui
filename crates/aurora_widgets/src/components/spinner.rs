use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

/// A loading spinner indicator.
///
/// Draws a circular track with a rotating arc. Animates automatically
/// without any external setup.
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
    start_time: Instant,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            diameter: 24.0,
            track_color: colors::muted(),
            color: colors::primary(),
            thickness: 3.0,
            start_time: Instant::now(),
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

        // Draw the background track ring
        let track_rect = Rect::new(
            cx - r - self.thickness / 2.0,
            cy - r - self.thickness / 2.0,
            cx + r + self.thickness / 2.0,
            cy + r + self.thickness / 2.0,
        );
        let corners = aurora_core::geometry::corners::Corners::all(r + self.thickness / 2.0);
        canvas.stroke_rounded_rect(track_rect, corners, self.thickness as u32, self.track_color);

        // Draw the rotating arc
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let speed = 1.5; // rotations per second
        let base_angle = elapsed * speed * std::f32::consts::TAU;
        let arc_sweep = std::f32::consts::TAU * 0.3; // 108-degree arc

        let segments = 24;
        let stroke = self.thickness as u32;
        for i in 0..segments {
            let t0 = i as f32 / segments as f32;
            let t1 = (i + 1) as f32 / segments as f32;
            let a0 = base_angle + t0 * arc_sweep;
            let a1 = base_angle + t1 * arc_sweep;

            let from = Point::new(cx + r * a0.cos(), cy + r * a0.sin());
            let to = Point::new(cx + r * a1.cos(), cy + r * a1.sin());
            canvas.draw_line(from, to, stroke, self.color);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }

    fn needs_animation(&self) -> bool {
        true
    }
}
