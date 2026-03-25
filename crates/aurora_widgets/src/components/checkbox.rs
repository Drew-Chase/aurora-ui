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

use super::colors;

/// A checkbox input with checked/unchecked state.
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
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
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
                colors::MUTED_FOREGROUND
            } else {
                colors::FOREGROUND
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
        let box_rect = Rect::new(
            rect.x1,
            rect.y1 + (rect.height() - self.size) / 2.0,
            rect.x1 + self.size,
            rect.y1 + (rect.height() - self.size) / 2.0 + self.size,
        );
        let corners = Corners::all(4.0);

        if self.checked {
            canvas.fill_rounded_rect(box_rect, corners, colors::PRIMARY);
            // Draw checkmark as two lines
            let x = box_rect.x1;
            let y = box_rect.y1;
            let s = self.size;
            canvas.draw_line(
                Point::new(x + s * 0.25, y + s * 0.5),
                Point::new(x + s * 0.45, y + s * 0.7),
                2,
                colors::PRIMARY_FOREGROUND,
            );
            canvas.draw_line(
                Point::new(x + s * 0.45, y + s * 0.7),
                Point::new(x + s * 0.75, y + s * 0.3),
                2,
                colors::PRIMARY_FOREGROUND,
            );
        } else {
            canvas.stroke_rounded_rect(box_rect, corners, 2, colors::INPUT_BORDER);
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
