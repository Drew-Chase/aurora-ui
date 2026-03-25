use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A toggle switch with on/off state.
///
/// # Example
/// ```ignore
/// Switch::new()
///     .checked(true)
///     .on_change(|on| println!("switch: {on}"))
/// ```
pub struct Switch {
    checked: bool,
    disabled: bool,
    track_width: f32,
    track_height: f32,
    on_color: Color,
    off_color: Color,
    thumb_color: Color,
    on_change: Option<Box<dyn FnMut(bool)>>,
}

impl Switch {
    pub fn new() -> Self {
        Self {
            checked: false,
            disabled: false,
            track_width: 44.0,
            track_height: 24.0,
            on_color: colors::PRIMARY,
            off_color: colors::INPUT_BORDER,
            thumb_color: Color::WHITE,
            on_change: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_color(mut self, color: Color) -> Self {
        self.on_color = color;
        self
    }

    pub fn off_color(mut self, color: Color) -> Self {
        self.off_color = color;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Switch {
    fn layout(&mut self, _available: Size, _ctx: &mut LayoutCtx) -> Size {
        Size::new(self.track_width, self.track_height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let track_corners = Corners::all(self.track_height / 2.0);
        let track_color = if self.checked {
            self.on_color
        } else {
            self.off_color
        };
        canvas.fill_rounded_rect(rect, track_corners, track_color);

        // Thumb
        let thumb_padding = 2.0;
        let thumb_size = self.track_height - thumb_padding * 2.0;
        let thumb_x = if self.checked {
            rect.x2 - thumb_size - thumb_padding
        } else {
            rect.x1 + thumb_padding
        };
        let thumb_rect = Rect::new(
            thumb_x,
            rect.y1 + thumb_padding,
            thumb_x + thumb_size,
            rect.y1 + thumb_padding + thumb_size,
        );
        let thumb_corners = Corners::all(thumb_size / 2.0);
        canvas.fill_rounded_rect(thumb_rect, thumb_corners, self.thumb_color);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if self.disabled {
            return EventResponse::default();
        }
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                self.checked = !self.checked;
                if let Some(ref mut cb) = self.on_change {
                    cb(self.checked);
                }
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                EventResponse {
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
}
