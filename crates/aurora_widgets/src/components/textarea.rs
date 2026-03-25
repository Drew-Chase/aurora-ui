use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000);

/// A multi-line text area input.
///
/// # Example
/// ```ignore
/// TextArea::new()
///     .placeholder("Type your message...")
///     .rows(5)
///     .on_change(|text| println!("text: {text}"))
/// ```
pub struct TextArea {
    id: u64,
    text: String,
    cursor_pos: usize,
    placeholder: String,
    rows: usize,
    background: Color,
    border_color: Color,
    text_color: Color,
    _placeholder_color: Color,
    corners: Corners,
    padding: Edges,
    font_size: f32,
    width: Option<f32>,
    height: Option<f32>,
    focused: bool,
    tab_index: Option<u32>,
    on_change: Option<Box<dyn FnMut(&str)>>,
    text_layout: Option<aurora_text::text_layout::TextLayout>,
    placeholder_layout: Option<aurora_text::text_layout::TextLayout>,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            text: String::new(),
            cursor_pos: 0,
            placeholder: String::new(),
            rows: 3,
            background: colors::background(),
            border_color: colors::input_border(),
            text_color: colors::foreground(),
            _placeholder_color: colors::muted_foreground(),
            corners: Corners::all(6.0),
            padding: Edges::new(8.0, 12.0, 8.0, 12.0),
            font_size: 14.0,
            width: None,
            height: None,
            focused: false,
            tab_index: None,
            on_change: None,
            text_layout: None,
            placeholder_layout: None,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor_pos = self.text.len();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
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

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
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

    pub fn tab_index(mut self, index: u32) -> Self {
        self.tab_index = Some(index);
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn notify_change(&mut self) {
        if let Some(ref mut cb) = self.on_change {
            cb(&self.text);
        }
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextArea {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let line_height = self.font_size * 1.4;
        let h = self.height.unwrap_or(
            self.padding.top + line_height * self.rows as f32 + self.padding.bottom,
        );
        let inner_w = w - self.padding.left - self.padding.right;

        // Text layout
        if !self.text.is_empty() {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(self.font_size);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.text, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.text_layout = Some(tl);
        } else {
            self.text_layout = None;
        }

        // Placeholder layout
        if !self.placeholder.is_empty() && self.text.is_empty() {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(self.font_size);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &self.placeholder, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.placeholder_layout = Some(tl);
        } else {
            self.placeholder_layout = None;
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background
        canvas.fill_rounded_rect(rect, self.corners, self.background);

        // Border
        let border_color = if self.focused { colors::ring() } else { self.border_color };
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Clip text area
        let text_rect = Rect::new(
            rect.x1 + self.padding.left,
            rect.y1 + self.padding.top,
            rect.x2 - self.padding.right,
            rect.y2 - self.padding.bottom,
        );
        canvas.push_clip(text_rect);

        let tx = text_rect.x1;
        let ty = text_rect.y1;

        // Draw text or placeholder
        if let Some(ref tl) = self.text_layout {
            canvas.draw_text(tl, tx as i32, ty as i32);
        } else if let Some(ref pl) = self.placeholder_layout {
            canvas.draw_text(pl, tx as i32, ty as i32);
        }

        // Cursor
        if self.focused {
            canvas.fill_rect(
                Rect::new(tx, ty, tx + 1.5, ty + self.font_size),
                self.text_color,
            );
        }

        canvas.pop_clip();
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
            WidgetEvent::Focus(target_id) => {
                if *target_id == self.id {
                    self.focused = true;
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
                    return EventResponse {
                        handled: true,
                        request_focus: Some(self.id),
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                if e.state == MouseState::Pressed && !rect.contains(&e.position) {
                    self.focused = false;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !self.focused {
                    return EventResponse::default();
                }
                self.text.insert(self.cursor_pos, *ch);
                self.cursor_pos += ch.len_utf8();
                self.notify_change();
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
                    Key::Enter => {
                        self.text.insert(self.cursor_pos, '\n');
                        self.cursor_pos += 1;
                        self.notify_change();
                    }
                    Key::Backspace => {
                        if self.cursor_pos > 0 {
                            let prev = self.text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            self.text.drain(prev..self.cursor_pos);
                            self.cursor_pos = prev;
                            self.notify_change();
                        }
                    }
                    Key::Delete => {
                        if self.cursor_pos < self.text.len() {
                            let next = self.text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.text.len());
                            self.text.drain(self.cursor_pos..next);
                            self.notify_change();
                        }
                    }
                    Key::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos = self.text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                        }
                    }
                    Key::Right => {
                        if self.cursor_pos < self.text.len() {
                            self.cursor_pos = self.text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.text.len());
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
}
