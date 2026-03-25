use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Orientation for the separator.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A visual divider between sections.
///
/// # Example
/// ```ignore
/// Separator::new()
///     .orientation(SeparatorOrientation::Horizontal)
///     .color(Color::new(228, 228, 231, 255))
/// ```
pub struct Separator {
    orientation: SeparatorOrientation,
    thickness: f32,
    color: Color,
    width: Option<f32>,
    height: Option<f32>,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            thickness: 1.0,
            color: colors::BORDER,
            width: None,
            height: None,
        }
    }

    pub fn horizontal() -> Self {
        Self::new().orientation(SeparatorOrientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new().orientation(SeparatorOrientation::Vertical)
    }

    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Separator {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        match self.orientation {
            SeparatorOrientation::Horizontal => Size::new(
                self.width.unwrap_or(available.width),
                self.height.unwrap_or(self.thickness),
            ),
            SeparatorOrientation::Vertical => Size::new(
                self.width.unwrap_or(self.thickness),
                self.height.unwrap_or(available.height),
            ),
        }
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        canvas.fill_rect(rect, self.color);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
