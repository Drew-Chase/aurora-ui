use crate::box_widget::BoxWidget;
use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

/// How a [`Positioned`] widget places its child.
pub enum Position {
    /// Normal flow — the child paints at the rect it receives from its parent.
    Relative,
    /// Offset from the parent's origin. Removed from normal flow
    /// ([`Size::zero()`] reported to parent).
    Absolute(Point),
    /// Offset from the window origin, ignoring the parent rect entirely.
    /// Removed from normal flow ([`Size::zero()`] reported to parent).
    Fixed(Point),
}

/// Wraps a single child and controls where it is painted.
///
/// Use inside a [`Stack`](super::stack::Stack) to layer children at
/// exact coordinates. `Absolute` and `Fixed` variants report zero size
/// during layout so they don't affect sibling positioning.
///
/// # Example
///
/// ```ignore
/// use aurora_ui::prelude::*;
/// use aurora_ui::aurora_widgets::layout::position::Positioned;
///
/// Positioned::fixed((20.0, 20.0))
///     .child(BoxWidget::new().width(50).height(50).background_color(Color::RED))
/// ```
pub struct Positioned {
    position: Position,
    width: Option<f32>,
    height: Option<f32>,
    child: Box<dyn Widget>,
    child_size: Size,
}

impl Default for Positioned {
    fn default() -> Self {
        Self {
            position: Position::Relative,
            width: None,
            height: None,
            child: Box::new(BoxWidget::new()),
            child_size: Size::zero(),
        }
    }
}

impl Positioned {
    /// Creates a positioned widget at an absolute offset from its parent's origin.
    pub fn absolute(position: impl Into<Point>) -> Self {
        Self {
            position: Position::Absolute(position.into()),
            ..Self::default()
        }
    }

    /// Creates a positioned widget in normal flow (no offset).
    pub fn relative() -> Self {
        Self {
            position: Position::Relative,
            ..Self::default()
        }
    }

    /// Creates a positioned widget at a fixed offset from the window origin.
    pub fn fixed(position: impl Into<Point>) -> Self {
        Self {
            position: Position::Fixed(position.into()),
            ..Self::default()
        }
    }

    /// Sets a fixed width override.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height override.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the child widget to position.
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Box::new(child);
        self
    }
}

impl Widget for Positioned {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let constrained = Size::new(
            self.width.unwrap_or(available.width),
            self.height.unwrap_or(available.height),
        );
        self.child_size = self.child.layout(constrained, ctx);
        match self.position {
            Position::Absolute(_) | Position::Fixed(_) => Size::zero(),
            Position::Relative => self.child_size,
        }
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        match self.position {
            Position::Relative => {
                self.child.paint(canvas, rect);
            }
            Position::Absolute(point) => {
                // Relative to parent's origin
                let positioned_rect = Rect::new(
                    rect.x1 + point.x,
                    rect.y1 + point.y,
                    rect.x1 + point.x + self.child_size.width,
                    rect.y1 + point.y + self.child_size.height,
                );
                self.child.paint(canvas, positioned_rect);
            }
            Position::Fixed(point) => {
                // Relative to window origin — ignore parent rect
                let positioned_rect = Rect::new(
                    point.x,
                    point.y,
                    point.x + self.child_size.width,
                    point.y + self.child_size.height,
                );
                self.child.paint(canvas, positioned_rect);
            }
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        let child_rect = match self.position {
            Position::Relative => rect,
            Position::Absolute(point) => Rect::new(
                rect.x1 + point.x,
                rect.y1 + point.y,
                rect.x1 + point.x + self.child_size.width,
                rect.y1 + point.y + self.child_size.height,
            ),
            Position::Fixed(point) => Rect::new(
                point.x,
                point.y,
                point.x + self.child_size.width,
                point.y + self.child_size.height,
            ),
        };
        self.child.paint_overlay(canvas, child_rect);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        std::slice::from_ref(&self.child)
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let child_rect = match self.position {
            Position::Relative => rect,
            Position::Absolute(point) => Rect::new(
                rect.x1 + point.x,
                rect.y1 + point.y,
                rect.x1 + point.x + self.child_size.width,
                rect.y1 + point.y + self.child_size.height,
            ),
            Position::Fixed(point) => Rect::new(
                point.x,
                point.y,
                point.x + self.child_size.width,
                point.y + self.child_size.height,
            ),
        };
        self.child.event(event, child_rect)
    }
}
