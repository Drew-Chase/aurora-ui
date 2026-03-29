use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(2_000_000);

/// OTP code input with individual character cells.
///
/// # Example
/// ```ignore
/// InputOtp::new(6)
///     .on_complete(|code| println!("code: {code}"))
/// ```
pub struct InputOtp {
    id: u64,
    length: usize,
    value: Vec<char>,
    cursor: usize,
    cell_size: f32,
    spacing: f32,
    background: Color,
    border_color: Color,
    focused_border: Color,
    corners: Corners,
    focused: bool,
    tab_index: Option<u32>,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str)>>,
    #[allow(clippy::type_complexity)]
    on_complete: Option<Box<dyn FnMut(&str)>>,
    char_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
}

impl InputOtp {
    pub fn new(length: usize) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            length: length.max(1),
            value: Vec::new(),
            cursor: 0,
            cell_size: 44.0,
            spacing: 8.0,
            background: colors::background(),
            border_color: colors::input_border(),
            focused_border: colors::ring(),
            corners: Corners::all(8.0),
            focused: false,
            tab_index: None,
            on_change: None,
            on_complete: None,
            char_layouts: Vec::new(),
        }
    }

    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell_size = size;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn tab_index(mut self, index: u32) -> Self {
        self.tab_index = Some(index);
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    pub fn on_complete(mut self, cb: impl FnMut(&str) + 'static) -> Self {
        self.on_complete = Some(Box::new(cb));
        self
    }

    fn value_string(&self) -> String {
        self.value.iter().collect()
    }

    fn notify_change(&mut self) {
        let s = self.value_string();
        if let Some(ref mut cb) = self.on_change {
            cb(&s);
        }
        if self.value.len() == self.length
            && let Some(ref mut cb) = self.on_complete
        {
            cb(&s);
        }
    }
}

impl Widget for InputOtp {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        let total_w = self.cell_size * self.length as f32
            + self.spacing * (self.length - 1) as f32;

        // Character layouts
        self.char_layouts.clear();
        for i in 0..self.length {
            if let Some(&ch) = self.value.get(i) {
                let text = ch.to_string();
                let mut opts = ctx.font_options.clone();
                opts.size = Some(20.0);
                opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
                let tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &text, &opts, colors::foreground(), None);
                self.char_layouts.push(Some(tl));
            } else {
                self.char_layouts.push(None);
            }
        }

        Size::new(total_w, self.cell_size)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let mut x = rect.x1;

        for i in 0..self.length {
            let cell_rect = Rect::new(x, rect.y1, x + self.cell_size, rect.y1 + self.cell_size);

            canvas.fill_rounded_rect(cell_rect, self.corners, self.background);

            let border = if self.focused && i == self.cursor {
                self.focused_border
            } else {
                self.border_color
            };
            canvas.stroke_rounded_rect(cell_rect, self.corners, 1, border);

            // Character
            if let Some(Some(tl)) = self.char_layouts.get(i) {
                let _s = tl.size(); let tw = _s.width; let th = _s.height;
                let tx = cell_rect.x1 + (self.cell_size - tw) / 2.0;
                let ty = cell_rect.y1 + (self.cell_size - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Cursor indicator (blinking dot)
            if self.focused && i == self.cursor && self.value.get(i).is_none() {
                let dot_size = 4.0;
                let dot_rect = Rect::new(
                    cell_rect.x1 + (self.cell_size - dot_size) / 2.0,
                    cell_rect.y1 + (self.cell_size - dot_size) / 2.0,
                    cell_rect.x1 + (self.cell_size + dot_size) / 2.0,
                    cell_rect.y1 + (self.cell_size + dot_size) / 2.0,
                );
                let dot_corners = Corners::all(dot_size / 2.0);
                canvas.fill_rounded_rect(dot_rect, dot_corners, colors::muted_foreground());
            }

            x += self.cell_size + self.spacing;
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn tab_index(&self) -> Option<u32> {
        self.tab_index
    }

    fn widget_id(&self) -> Option<u64> {
        Some(self.id)
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Focus(target_id, _select_all) => {
                if *target_id == self.id {
                    self.focused = true;
                    self.cursor = self.value.len().min(self.length - 1);
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Pressed && rect.contains(&e.position) {
                    self.focused = true;
                    // Determine which cell was clicked
                    let relative_x = e.position.x - rect.x1;
                    let cell_with_spacing = self.cell_size + self.spacing;
                    let idx = (relative_x / cell_with_spacing) as usize;
                    self.cursor = idx.min(self.value.len()).min(self.length - 1);
                    return EventResponse {
                        handled: true,
                        request_focus: Some(self.id),
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                if e.state == MouseState::Pressed {
                    self.focused = false;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                EventResponse {
                    cursor: Some(CursorIcon::Text),
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !self.focused || ch.is_control() {
                    return EventResponse::default();
                }
                if self.value.len() < self.length {
                    if self.cursor < self.value.len() {
                        self.value[self.cursor] = *ch;
                    } else {
                        self.value.push(*ch);
                    }
                    self.cursor = (self.cursor + 1).min(self.length - 1);
                    self.notify_change();
                }
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::KeyPressed { key, modifiers }) => {
                if !self.focused {
                    return EventResponse::default();
                }
                match key {
                    Key::Backspace => {
                        if !self.value.is_empty() {
                            if self.cursor < self.value.len() {
                                self.value.remove(self.cursor);
                            } else {
                                self.value.pop();
                            }
                            if self.cursor > 0 {
                                self.cursor -= 1;
                            }
                            self.notify_change();
                        }
                    }
                    Key::Left => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    Key::Right => {
                        if self.cursor < self.length - 1 {
                            self.cursor += 1;
                        }
                    }
                    Key::Tab => {
                        self.focused = false;
                        return EventResponse {
                            handled: true,
                            request_focus: Some(self.id),
                            focus_next: !modifiers.shift,
                            focus_prev: modifiers.shift,
                            ..Default::default()
                        };
                    }
                    _ => return EventResponse::default(),
                }
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group).with_label("One-time password".to_string())    }
}
