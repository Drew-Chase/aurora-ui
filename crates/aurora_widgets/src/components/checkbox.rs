use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontWeight;
use aurora_text::text_layout::TextLayout;
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.15; // seconds

/// A checkbox input with checked/unchecked state.
///
/// The checkmark and background animate smoothly on toggle.
///
/// # Example
/// ```ignore
/// Checkbox::new()
///     .checked(true)
///     .label("Accept terms")
///     .on_change(|checked| println!("checked: {checked}"))
/// ```
pub struct Checkbox {
    checked: bool,
    label: Option<String>,
    size: f32,
    disabled: bool,
    on_change: Option<Box<dyn FnMut(bool)>>,
    label_layout: Option<TextLayout>,
    /// 0.0 = unchecked, 1.0 = checked
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            checked: false,
            label: None,
            size: 18.0,
            disabled: false,
            on_change: None,
            label_layout: None,
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        let t = if checked { 1.0 } else { 0.0 };
        self.anim_from = t;
        self.anim_to = t;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Checkbox {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let mut w = self.size;
        let mut h = self.size;

        if let Some(ref label) = self.label {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(FontWeight::Normal);
            let label_color = if self.disabled {
                colors::muted_foreground()
            } else {
                colors::foreground()
            };
            let max_label_w = available.width - self.size - 8.0;
            let mut tl = TextLayout::new(ctx.font_manager, label, &opts, label_color, None);
            tl.set_max_width(ctx.font_manager, max_label_w.max(0.0));
            let ts = tl.size();
            w = self.size + 8.0 + ts.width;
            h = h.max(ts.height);
            self.label_layout = Some(tl);
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let t = self.current_t();
        let box_rect = Rect::new(
            rect.x1,
            rect.y1 + (rect.height() - self.size) / 2.0,
            rect.x1 + self.size,
            rect.y1 + (rect.height() - self.size) / 2.0 + self.size,
        );
        let corners = Corners::all(4.0);

        if t > 0.0 {
            // Interpolate from border-only to filled primary
            let bg = colors::background().lerp(&colors::primary(), t);
            canvas.fill_rounded_rect(box_rect, corners, bg);

            // Draw checkmark scaled by t (from center)
            let x = box_rect.x1;
            let y = box_rect.y1;
            let s = self.size;
            let cx = x + s * 0.5;
            let cy = y + s * 0.5;

            // Checkmark control points, offset from center, scaled by t
            let p1 = Point::new(cx + s * -0.25 * t, cy);
            let p2 = Point::new(cx + s * -0.05 * t, cy + s * 0.2 * t);
            let p3 = Point::new(cx + s * 0.25 * t, cy + s * -0.2 * t);

            let check_alpha = (t * 255.0).min(255.0) as u8;
            let check_color = aurora_core::color::Color::new(
                colors::primary_foreground().red,
                colors::primary_foreground().green,
                colors::primary_foreground().blue,
                check_alpha,
            );
            canvas.draw_line(p1, p2, 2, check_color);
            canvas.draw_line(p2, p3, 2, check_color);
        }

        if t < 1.0 {
            // Draw border that fades out as we fill
            let border_alpha = ((1.0 - t) * 255.0).min(255.0) as u8;
            let border_color = aurora_core::color::Color::new(
                colors::input_border().red,
                colors::input_border().green,
                colors::input_border().blue,
                border_alpha,
            );
            canvas.stroke_rounded_rect(box_rect, corners, 2, border_color);
        }

        if let Some(ref tl) = self.label_layout {
            let ts = tl.size();
            let label_x = rect.x1 + self.size + 8.0;
            let label_y = rect.y1 + (rect.height() - ts.height) / 2.0;
            canvas.draw_text(tl, label_x as i32, label_y as i32);
        }
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
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }
}
