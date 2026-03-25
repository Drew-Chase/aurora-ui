use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Popover placement relative to the trigger.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PopoverSide {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

/// A click-triggered popup panel that appears near the trigger widget.
///
/// # Example
/// ```ignore
/// Popover::new()
///     .trigger(Label::new("Click me"))
///     .content(Label::new("Popover content here"))
///     .side(PopoverSide::Bottom)
/// ```
pub struct Popover {
    trigger: Option<Box<dyn Widget>>,
    content: Option<Box<dyn Widget>>,
    open: bool,
    side: PopoverSide,
    offset: f32,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    trigger_size: Size,
    content_size: Size,
}

impl Popover {
    pub fn new() -> Self {
        Self {
            trigger: None,
            content: None,
            open: false,
            side: PopoverSide::Bottom,
            offset: 4.0,
            background: colors::POPOVER,
            border_color: colors::BORDER,
            corners: Corners::all(8.0),
            padding: Edges::new(12.0, 12.0, 12.0, 12.0),
            trigger_size: Size::default(),
            content_size: Size::default(),
        }
    }

    pub fn trigger(mut self, widget: impl Widget + 'static) -> Self {
        self.trigger = Some(Box::new(widget));
        self
    }

    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
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

    fn popover_rect(&self, trigger_rect: &Rect) -> Rect {
        let pw = self.content_size.width + self.padding.left + self.padding.right;
        let ph = self.content_size.height + self.padding.top + self.padding.bottom;
        let cx = trigger_rect.x1 + trigger_rect.width() / 2.0;

        match self.side {
            PopoverSide::Bottom => Rect::new(
                cx - pw / 2.0,
                trigger_rect.y2 + self.offset,
                cx + pw / 2.0,
                trigger_rect.y2 + self.offset + ph,
            ),
            PopoverSide::Top => Rect::new(
                cx - pw / 2.0,
                trigger_rect.y1 - ph - self.offset,
                cx + pw / 2.0,
                trigger_rect.y1 - self.offset,
            ),
            PopoverSide::Left => {
                let cy = trigger_rect.y1 + trigger_rect.height() / 2.0;
                Rect::new(
                    trigger_rect.x1 - pw - self.offset,
                    cy - ph / 2.0,
                    trigger_rect.x1 - self.offset,
                    cy + ph / 2.0,
                )
            }
            PopoverSide::Right => {
                let cy = trigger_rect.y1 + trigger_rect.height() / 2.0;
                Rect::new(
                    trigger_rect.x2 + self.offset,
                    cy - ph / 2.0,
                    trigger_rect.x2 + self.offset + pw,
                    cy + ph / 2.0,
                )
            }
        }
    }
}

impl Default for Popover {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Popover {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        if let Some(ref mut trigger) = self.trigger {
            self.trigger_size = trigger.layout(available, ctx);
        }

        if self.open {
            if let Some(ref mut content) = self.content {
                let content_available = Size::new(
                    (available.width - self.padding.left - self.padding.right).max(0.0),
                    f32::MAX,
                );
                self.content_size = content.layout(content_available, ctx);
            }
        }

        self.trigger_size
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Paint trigger
        if let Some(ref trigger) = self.trigger {
            trigger.paint(canvas, rect);
        }

        // Paint popover when open
        if self.open {
            let pr = self.popover_rect(&rect);
            canvas.fill_rounded_rect(pr, self.corners, self.background);
            canvas.stroke_rounded_rect(pr, self.corners, 1, self.border_color);

            if let Some(ref content) = self.content {
                let content_rect = Rect::new(
                    pr.x1 + self.padding.left,
                    pr.y1 + self.padding.top,
                    pr.x1 + self.padding.left + self.content_size.width,
                    pr.y1 + self.padding.top + self.content_size.height,
                );
                content.paint(canvas, content_rect);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if rect.contains(&e.position) {
                    self.open = !self.open;
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                // Click outside popover closes it
                if self.open {
                    let pr = self.popover_rect(&rect);
                    if !pr.contains(&e.position) {
                        self.open = false;
                        return EventResponse {
                            handled: true,
                            ..Default::default()
                        };
                    }
                    // Forward to content
                    if let Some(ref mut content) = self.content {
                        let content_rect = Rect::new(
                            pr.x1 + self.padding.left,
                            pr.y1 + self.padding.top,
                            pr.x1 + self.padding.left + self.content_size.width,
                            pr.y1 + self.padding.top + self.content_size.height,
                        );
                        return content.event(event, content_rect);
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.open {
                    let pr = self.popover_rect(&rect);
                    if let Some(ref mut content) = self.content {
                        let content_rect = Rect::new(
                            pr.x1 + self.padding.left,
                            pr.y1 + self.padding.top,
                            pr.x1 + self.padding.left + self.content_size.width,
                            pr.y1 + self.padding.top + self.content_size.height,
                        );
                        return content.event(event, content_rect);
                    }
                }
                EventResponse::default()
            }
            _ => {
                if self.open {
                    let pr = self.popover_rect(&rect);
                    if let Some(ref mut content) = self.content {
                        let content_rect = Rect::new(
                            pr.x1 + self.padding.left,
                            pr.y1 + self.padding.top,
                            pr.x1 + self.padding.left + self.content_size.width,
                            pr.y1 + self.padding.top + self.content_size.height,
                        );
                        return content.event(event, content_rect);
                    }
                }
                EventResponse::default()
            }
        }
    }
}
