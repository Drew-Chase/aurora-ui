use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.15; // seconds

/// A toggle switch with on/off state.
///
/// The thumb slides smoothly when toggled and the track color transitions.
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
    /// 0.0 = fully off, 1.0 = fully on
    anim_progress: f32,
    anim_start: Instant,
    anim_from: f32,
    anim_to: f32,
}

impl Switch {
    pub fn new() -> Self {
        Self {
            checked: false,
            disabled: false,
            track_width: 44.0,
            track_height: 24.0,
            on_color: colors::primary(),
            off_color: colors::input_border(),
            thumb_color: colors::background(),
            on_change: None,
            anim_progress: 0.0,
            anim_start: Instant::now(),
            anim_from: 0.0,
            anim_to: 0.0,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        let t = if checked { 1.0 } else { 0.0 };
        self.anim_progress = t;
        self.anim_from = t;
        self.anim_to = t;
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

    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        // Ease-out cubic
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
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
        let t = self.current_t();

        // Interpolate track color
        let track_color = Color::lerp(&self.off_color, &self.on_color, t);
        let track_corners = Corners::all(self.track_height / 2.0);
        canvas.fill_rounded_rect(rect, track_corners, track_color);

        // Thumb slides from left to right
        let thumb_padding = 2.0;
        let thumb_size = self.track_height - thumb_padding * 2.0;
        let left_x = rect.x1 + thumb_padding;
        let right_x = rect.x2 - thumb_size - thumb_padding;
        let thumb_x = left_x + (right_x - left_x) * t;

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
                // Start animation from current visual position
                self.anim_from = self.current_t();
                self.anim_to = if self.checked { 1.0 } else { 0.0 };
                self.anim_start = Instant::now();

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

    fn needs_animation(&self) -> bool {
        (self.anim_progress - self.anim_to).abs() > f32::EPSILON
            || self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Switch).with_checked(self.checked)    }
}
