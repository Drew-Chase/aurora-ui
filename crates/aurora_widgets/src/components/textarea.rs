use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::undo::UndoStack;
use aurora_render::canvas::Canvas;

/// Tracks the last mutation type for undo grouping.
#[derive(PartialEq)]
enum LastAction {
    None,
    Typing,
    Other,
}

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
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str)>>,
    text_layout: Option<aurora_text::text_layout::TextLayout>,
    placeholder_layout: Option<aurora_text::text_layout::TextLayout>,
    error: bool,
    disabled: bool,
    undo_stack: UndoStack<(String, usize)>,
    last_action: LastAction,
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
            error: false,
            disabled: false,
            undo_stack: UndoStack::new(),
            last_action: LastAction::None,
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

    /// Enables or disables the error state (red border).
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Enables or disables the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        let h = self
            .height
            .unwrap_or(self.padding.top + line_height * self.rows as f32 + self.padding.bottom);
        let inner_w = w - self.padding.left - self.padding.right;

        // Use muted foreground color when disabled
        let text_color = if self.disabled {
            colors::muted_foreground()
        } else {
            colors::foreground()
        };

        // Text layout
        if !self.text.is_empty() {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(self.font_size);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &self.text,
                &opts,
                text_color,
                None,
            );
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
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &self.placeholder,
                &opts,
                text_color,
                None,
            );
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.placeholder_layout = Some(tl);
        } else {
            self.placeholder_layout = None;
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background — use muted color when disabled for a faded look
        let bg = if self.disabled {
            colors::muted()
        } else {
            self.background
        };
        canvas.fill_rounded_rect(rect, self.corners, bg);

        // Border (error > focused > default)
        let border_color = if self.error {
            colors::destructive()
        } else if self.focused && !self.disabled {
            colors::ring()
        } else {
            self.border_color
        };
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

        // Cursor — don't show cursor when disabled
        if self.focused && !self.disabled {
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
        if self.disabled {
            return EventResponse::default();
        }
        match event {
            WidgetEvent::Focus(target_id, _select_all) => {
                if *target_id == self.id {
                    self.focused = true;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Pressed && rect.contains(&e.position) {
                    self.focused = true;
                    return EventResponse {
                        status: EventStatus::Consumed,
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
                if self.last_action != LastAction::Typing {
                    self.undo_stack.push((self.text.clone(), self.cursor_pos));
                }
                self.last_action = LastAction::Typing;
                self.text.insert(self.cursor_pos, *ch);
                self.cursor_pos += ch.len_utf8();
                self.notify_change();
                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::KeyPressed { key, modifiers }) => {
                if !self.focused {
                    return EventResponse::default();
                }

                // Ctrl+Z — undo
                if modifiers.ctrl && !modifiers.shift && *key == Key::Character('z') {
                    let current = (self.text.clone(), self.cursor_pos);
                    if let Some((text, pos)) = self.undo_stack.undo(current) {
                        self.text = text;
                        self.cursor_pos = pos;
                        self.last_action = LastAction::None;
                        self.notify_change();
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                // Ctrl+Shift+Z or Ctrl+Y — redo
                if (modifiers.ctrl && modifiers.shift && *key == Key::Character('z'))
                    || (modifiers.ctrl && *key == Key::Character('y'))
                {
                    let current = (self.text.clone(), self.cursor_pos);
                    if let Some((text, pos)) = self.undo_stack.redo(current) {
                        self.text = text;
                        self.cursor_pos = pos;
                        self.last_action = LastAction::None;
                        self.notify_change();
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                match key {
                    Key::Enter => {
                        if self.last_action != LastAction::Typing {
                            self.undo_stack.push((self.text.clone(), self.cursor_pos));
                        }
                        self.last_action = LastAction::Typing;
                        self.text.insert(self.cursor_pos, '\n');
                        self.cursor_pos += 1;
                        self.notify_change();
                    }
                    Key::Backspace => {
                        self.undo_stack.push((self.text.clone(), self.cursor_pos));
                        self.last_action = LastAction::Other;
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
                        self.undo_stack.push((self.text.clone(), self.cursor_pos));
                        self.last_action = LastAction::Other;
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
                            status: EventStatus::Consumed,
                            request_focus: Some(self.id),
                            focus_next: !modifiers.shift,
                            focus_prev: modifiers.shift,
                            ..Default::default()
                        };
                    }
                    _ => return EventResponse::default(),
                }
                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        let mut info = aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::MultilineTextInput);
        if !self.placeholder.is_empty() {
            info = info.with_label(self.placeholder.clone());
        }
        if !self.text.is_empty() {
            info = info.with_value(self.text.clone());
        }
        info.multiline = true;
        info
    }
}
