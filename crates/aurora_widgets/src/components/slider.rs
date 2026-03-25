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

/// A slider input for selecting a value within a range.
///
/// # Example
/// ```ignore
/// Slider::new()
///     .value(0.5)
///     .min(0.0)
///     .max(100.0)
///     .on_change(|val| println!("value: {val}"))
/// ```
pub struct Slider {
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    track_height: f32,
    thumb_size: f32,
    track_color: Color,
    fill_color: Color,
    thumb_color: Color,
    width: Option<f32>,
    disabled: bool,
    dragging: bool,
    on_change: Option<Box<dyn FnMut(f32)>>,
}

impl Slider {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: None,
            track_height: 6.0,
            thumb_size: 20.0,
            track_color: colors::SECONDARY,
            fill_color: colors::PRIMARY,
            thumb_color: Color::WHITE,
            width: None,
            disabled: false,
            dragging: false,
            on_change: None,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(f32) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn normalized(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    fn value_from_x(&self, x: f32, rect: &Rect) -> f32 {
        let track_x = rect.x1 + self.thumb_size / 2.0;
        let track_w = rect.width() - self.thumb_size;
        if track_w <= 0.0 {
            return self.min;
        }
        let t = ((x - track_x) / track_w).clamp(0.0, 1.0);
        let mut val = self.min + t * (self.max - self.min);
        if let Some(step) = self.step {
            val = (val / step).round() * step;
        }
        val.clamp(self.min, self.max)
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Slider {
    fn layout(&mut self, available: Size, _ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        Size::new(w, self.thumb_size)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let track_y = rect.y1 + (rect.height() - self.track_height) / 2.0;
        let track_rect = Rect::new(
            rect.x1 + self.thumb_size / 2.0,
            track_y,
            rect.x2 - self.thumb_size / 2.0,
            track_y + self.track_height,
        );
        let track_corners = Corners::all(self.track_height / 2.0);

        // Track background
        canvas.fill_rounded_rect(track_rect, track_corners, self.track_color);

        // Filled portion
        let norm = self.normalized();
        let fill_w = track_rect.width() * norm;
        if fill_w > 0.0 {
            let fill_rect = Rect::new(track_rect.x1, track_rect.y1, track_rect.x1 + fill_w, track_rect.y2);
            canvas.fill_rounded_rect(fill_rect, track_corners, self.fill_color);
        }

        // Thumb
        let thumb_x = track_rect.x1 + fill_w - self.thumb_size / 2.0;
        let thumb_rect = Rect::new(
            thumb_x,
            rect.y1,
            thumb_x + self.thumb_size,
            rect.y1 + self.thumb_size,
        );
        let thumb_corners = Corners::all(self.thumb_size / 2.0);
        canvas.fill_rounded_rect(thumb_rect, thumb_corners, self.thumb_color);
        canvas.stroke_rounded_rect(thumb_rect, thumb_corners, 1, colors::BORDER);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if self.disabled {
            return EventResponse::default();
        }
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) if rect.contains(&e.position) => {
                if e.state == MouseState::Pressed {
                    self.dragging = true;
                    self.value = self.value_from_x(e.position.x, &rect);
                    if let Some(ref mut cb) = self.on_change {
                        cb(self.value);
                    }
                } else {
                    self.dragging = false;
                }
                EventResponse {
                    handled: true,
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.dragging {
                    self.value = self.value_from_x(pos.x, &rect);
                    if let Some(ref mut cb) = self.on_change {
                        cb(self.value);
                    }
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Released =>
            {
                self.dragging = false;
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}
