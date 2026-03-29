use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A progress bar showing completion percentage.
///
/// # Example
/// ```ignore
/// Progress::new()
///     .value(0.65)
///     .color(Color::new(59, 130, 246, 255))
/// ```
pub struct Progress {
    value: f32,
    track_color: Color,
    fill_color: Color,
    height: f32,
    width: Option<f32>,
    corners: Corners,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            track_color: colors::secondary(),
            fill_color: colors::primary(),
            height: 8.0,
            width: None,
            corners: Corners::all(9999.0),
        }
    }

    /// Set the progress value (0.0 to 1.0).
    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Progress {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        Size::new(self.width.unwrap_or(available.width), self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Track
        canvas.fill_rounded_rect(rect, self.corners, self.track_color);

        // Fill
        if self.value > 0.0 {
            let fill_width = rect.width() * self.value;
            let fill_rect = Rect::new(rect.x1, rect.y1, rect.x1 + fill_width, rect.y2);
            canvas.fill_rounded_rect(fill_rect, self.corners, self.fill_color);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::ProgressIndicator)
            .with_numeric_value(self.value as f64)
            .with_numeric_range(0.0, 1.0)
    }
}
