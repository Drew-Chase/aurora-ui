use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// Direction of the resizable handle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ResizeDirection {
    #[default]
    Right,
    Bottom,
    Left,
}

/// A resizable panel container. The user can drag a handle to resize the panel.
///
/// # Example
/// ```ignore
/// Resizable::new()
///     .child(Label::new("Resizable content"))
///     .direction(ResizeDirection::Right)
///     .initial_size(300.0)
///     .min_size(100.0)
///     .max_size(600.0)
/// ```
pub struct Resizable {
    child: Option<Box<dyn Widget>>,
    direction: ResizeDirection,
    current_size: f32,
    min_size: f32,
    max_size: f32,
    handle_width: f32,
    handle_color: Color,
    handle_hover_color: Color,
    dragging: bool,
    drag_start: f32,
    drag_start_size: f32,
    hovered: bool,
    child_size: Size,
}

impl Resizable {
    pub fn new() -> Self {
        Self {
            child: None,
            direction: ResizeDirection::Right,
            current_size: 300.0,
            min_size: 50.0,
            max_size: f32::MAX,
            handle_width: 4.0,
            handle_color: colors::border(),
            handle_hover_color: colors::primary(),
            dragging: false,
            drag_start: 0.0,
            drag_start_size: 0.0,
            hovered: false,
            child_size: Size::default(),
        }
    }

    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    pub fn direction(mut self, direction: ResizeDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn initial_size(mut self, size: f32) -> Self {
        self.current_size = size;
        self
    }

    pub fn min_size(mut self, size: f32) -> Self {
        self.min_size = size;
        self
    }

    pub fn max_size(mut self, size: f32) -> Self {
        self.max_size = size;
        self
    }

    pub fn handle_width(mut self, width: f32) -> Self {
        self.handle_width = width;
        self
    }

    fn handle_rect(&self, rect: &Rect) -> Rect {
        match self.direction {
            ResizeDirection::Right => Rect::new(
                rect.x1 + self.current_size - self.handle_width / 2.0,
                rect.y1,
                rect.x1 + self.current_size + self.handle_width / 2.0,
                rect.y2,
            ),
            ResizeDirection::Bottom => Rect::new(
                rect.x1,
                rect.y1 + self.current_size - self.handle_width / 2.0,
                rect.x2,
                rect.y1 + self.current_size + self.handle_width / 2.0,
            ),
            ResizeDirection::Left => Rect::new(
                rect.x2 - self.current_size - self.handle_width / 2.0,
                rect.y1,
                rect.x2 - self.current_size + self.handle_width / 2.0,
                rect.y2,
            ),
        }
    }

    fn content_rect(&self, rect: &Rect) -> Rect {
        match self.direction {
            ResizeDirection::Right => Rect::new(
                rect.x1,
                rect.y1,
                rect.x1 + self.current_size - self.handle_width,
                rect.y2,
            ),
            ResizeDirection::Bottom => Rect::new(
                rect.x1,
                rect.y1,
                rect.x2,
                rect.y1 + self.current_size - self.handle_width,
            ),
            ResizeDirection::Left => Rect::new(
                rect.x2 - self.current_size + self.handle_width,
                rect.y1,
                rect.x2,
                rect.y2,
            ),
        }
    }
}

impl Default for Resizable {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Resizable {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        self.current_size = self.current_size.clamp(self.min_size, self.max_size);

        let (w, h) = match self.direction {
            ResizeDirection::Right | ResizeDirection::Left => (self.current_size, available.height),
            ResizeDirection::Bottom => (available.width, self.current_size),
        };

        if let Some(ref mut child) = self.child {
            let content_w = if matches!(
                self.direction,
                ResizeDirection::Right | ResizeDirection::Left
            ) {
                (w - self.handle_width).max(0.0)
            } else {
                w
            };
            let content_h = if matches!(self.direction, ResizeDirection::Bottom) {
                (h - self.handle_width).max(0.0)
            } else {
                available.height
            };
            let child_available = Size::new(content_w, content_h);
            self.child_size = child.layout(child_available, ctx);
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Paint child
        if let Some(ref child) = self.child {
            let cr = self.content_rect(&rect);
            canvas.push_clip(cr);
            let child_rect = Rect::new(
                cr.x1,
                cr.y1,
                cr.x1 + self.child_size.width,
                cr.y1 + self.child_size.height,
            );
            child.paint(canvas, child_rect);
            canvas.pop_clip();
        }

        // Paint handle
        let hr = self.handle_rect(&rect);
        let color = if self.dragging || self.hovered {
            self.handle_hover_color
        } else {
            self.handle_color
        };
        canvas.fill_rect(hr, color);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        let hr = self.handle_rect(&rect);
        let resize_cursor = match self.direction {
            ResizeDirection::Right | ResizeDirection::Left => CursorIcon::Grab,
            ResizeDirection::Bottom => CursorIcon::Grab,
        };

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Pressed && hr.contains(&e.position) {
                    self.dragging = true;
                    self.drag_start = match self.direction {
                        ResizeDirection::Right | ResizeDirection::Left => e.position.x,
                        ResizeDirection::Bottom => e.position.y,
                    };
                    self.drag_start_size = self.current_size;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(resize_cursor),
                        ..Default::default()
                    };
                }
                if e.state == MouseState::Released && self.dragging {
                    self.dragging = false;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }
                // Forward to child
                let cr = self.content_rect(&rect);
                if let Some(ref mut child) = self.child {
                    return child.event(event, cr);
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.dragging {
                    let delta = match self.direction {
                        ResizeDirection::Right => pos.x - self.drag_start,
                        ResizeDirection::Bottom => pos.y - self.drag_start,
                        ResizeDirection::Left => self.drag_start - pos.x,
                    };
                    self.current_size =
                        (self.drag_start_size + delta).clamp(self.min_size, self.max_size);
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(resize_cursor),
                        ..Default::default()
                    };
                }
                self.hovered = hr.contains(pos);
                if self.hovered {
                    return EventResponse {
                        cursor: Some(resize_cursor),
                        ..Default::default()
                    };
                }
                let cr = self.content_rect(&rect);
                if let Some(ref mut child) = self.child {
                    return child.event(event, cr);
                }
                EventResponse::default()
            }
            _ => {
                let cr = self.content_rect(&rect);
                if let Some(ref mut child) = self.child {
                    return child.event(event, cr);
                }
                EventResponse::default()
            }
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Splitter)
    }
}
