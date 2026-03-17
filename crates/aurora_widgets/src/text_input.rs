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
use aurora_text::font_options::FontOptions;
use aurora_text::text_layout::TextLayout;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A single-line text input field.
///
/// Supports keyboard input, cursor navigation (arrow keys, Home, End),
/// backspace/delete, click-to-focus, and click-to-position-cursor.
///
/// # Example
///
/// ```no_run
/// use aurora_ui::prelude::*;
///
/// TextInput::new()
///     .placeholder("Enter your name")
///     .font_size(16.0)
///     .padding(Edges::symmetric(8.0, 12.0))
///     .on_change(|text| println!("Input: {text}"))
/// ```
pub struct TextInput {
    id: u64,
    text: String,
    cursor_pos: usize,
    font: FontOptions,
    color: Color,
    placeholder_color: Color,
    background: Color,
    focused_background: Color,
    corners: Corners,
    padding: Edges,
    placeholder: String,
    focused: bool,
    width: Option<f32>,
    height: Option<f32>,
    text_layout: Option<TextLayout>,
    cursor_layout: Option<TextLayout>,
    placeholder_layout: Option<TextLayout>,
    cursor_pixel_x: f32,
    on_change: Option<Box<dyn FnMut(&str)>>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            text: String::new(),
            cursor_pos: 0,
            font: FontOptions::default(),
            color: Color::BLACK,
            placeholder_color: Color::new(160, 160, 160, 255),
            background: Color::new(245, 245, 245, 255),
            focused_background: Color::WHITE,
            corners: Corners::all(4.0),
            padding: Edges::symmetric(6.0, 10.0),
            placeholder: String::new(),
            focused: false,
            width: None,
            height: None,
            text_layout: None,
            cursor_layout: None,
            placeholder_layout: None,
            cursor_pixel_x: 0.0,
            on_change: None,
        }
    }
}

impl TextInput {
    /// Creates a new empty text input.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial text content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor_pos = self.text.len();
        self
    }

    /// Sets the placeholder text shown when the input is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the font size in pixels.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font.size = Some(size);
        self
    }

    /// Sets the font family.
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font.family = Some(family.into());
        self
    }

    /// Sets the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the placeholder text color.
    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = color;
        self
    }

    /// Sets the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Sets the background color when focused.
    pub fn focused_background(mut self, color: Color) -> Self {
        self.focused_background = color;
        self
    }

    /// Sets the corner radii.
    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    /// Sets the padding inside the input.
    pub fn padding(mut self, padding: impl Into<Edges>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets a fixed width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a fixed height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Registers a callback invoked whenever the text content changes.
    pub fn on_change(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    fn notify_change(&mut self) {
        if let Some(ref mut on_change) = self.on_change {
            on_change(&self.text);
        }
    }
}

impl Widget for TextInput {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let resolved = self.font.resolve(ctx.font_options);
        let font_size = resolved.effective_size();

        let w = self.width.unwrap_or(available.width).min(available.width);
        let h = self
            .height
            .unwrap_or(font_size + self.padding.vertical())
            .min(available.height);
        let max_width = (w - self.padding.horizontal()).max(0.0);

        // Build text layout
        if !self.text.is_empty() {
            let mut tl =
                TextLayout::new(ctx.font_manager, &self.text, &resolved, self.color, None);
            tl.set_max_width(ctx.font_manager, max_width);
            self.text_layout = Some(tl);
        } else {
            self.text_layout = None;
        }

        // Build placeholder layout
        if !self.placeholder.is_empty() && self.text.is_empty() {
            let mut pl = TextLayout::new(
                ctx.font_manager,
                &self.placeholder,
                &resolved,
                self.placeholder_color,
                None,
            );
            pl.set_max_width(ctx.font_manager, max_width);
            self.placeholder_layout = Some(pl);
        } else {
            self.placeholder_layout = None;
        }

        // Measure cursor x position
        if self.cursor_pos > 0 && self.cursor_pos <= self.text.len() {
            let before = &self.text[..self.cursor_pos];
            let cl = TextLayout::new(ctx.font_manager, before, &resolved, self.color, None);
            self.cursor_pixel_x = cl.size().width;
            self.cursor_layout = Some(cl);
        } else {
            self.cursor_pixel_x = 0.0;
            self.cursor_layout = None;
        }

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let bg = if self.focused {
            self.focused_background
        } else {
            self.background
        };
        canvas.fill_rounded_rect(rect, self.corners, bg);

        // Draw text or placeholder
        let text_x = (rect.x1 + self.padding.left) as i32;
        let text_y = (rect.y1 + self.padding.top) as i32;

        if let Some(ref tl) = self.text_layout {
            canvas.draw_text(tl, text_x, text_y);
        } else if let Some(ref pl) = self.placeholder_layout {
            canvas.draw_text(pl, text_x, text_y);
        }

        // Draw cursor
        if self.focused {
            let font_size = self.font.effective_size();
            let cursor_x = rect.x1 + self.padding.left + self.cursor_pixel_x;
            let cursor_y = rect.y1 + self.padding.top;
            canvas.fill_rect(
                Rect::new(cursor_x, cursor_y, cursor_x + 1.5, cursor_y + font_size),
                self.color,
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(click)) => {
                if click.state == MouseState::Pressed && rect.contains(&click.position) {
                    self.focused = true;

                    // Position cursor based on click x
                    let click_x = click.position.x - rect.x1 - self.padding.left;
                    if let Some(ref tl) = self.text_layout {
                        let text_width = tl.size().width;
                        if click_x >= text_width {
                            self.cursor_pos = self.text.len();
                        } else {
                            // Walk characters to find closest position
                            let mut best_pos = 0;
                            let mut best_dist = click_x.abs();
                            let mut running_width = 0.0f32;
                            for (i, ch) in self.text.char_indices() {
                                let next = i + ch.len_utf8();
                                // Approximate: scale linearly
                                let char_width =
                                    text_width * (ch.len_utf8() as f32 / self.text.len() as f32);
                                running_width += char_width;
                                let dist = (click_x - running_width).abs();
                                if dist < best_dist {
                                    best_dist = dist;
                                    best_pos = next;
                                }
                            }
                            self.cursor_pos = best_pos;
                        }
                    } else {
                        self.cursor_pos = 0;
                    }

                    return EventResponse {
                        handled: true,
                        request_focus: Some(self.id),
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                if click.state == MouseState::Pressed && !rect.contains(&click.position) {
                    self.focused = false;
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    EventResponse {
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    }
                } else {
                    EventResponse::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !self.focused || ch.is_control() {
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
                        if modifiers.ctrl {
                            // Jump to previous word boundary
                            self.cursor_pos = prev_word_boundary(&self.text, self.cursor_pos);
                        } else if self.cursor_pos > 0 {
                            self.cursor_pos = self.text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                        }
                    }
                    Key::Right => {
                        if modifiers.ctrl {
                            self.cursor_pos = next_word_boundary(&self.text, self.cursor_pos);
                        } else if self.cursor_pos < self.text.len() {
                            self.cursor_pos = self.text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.text.len());
                        }
                    }
                    Key::Home => self.cursor_pos = 0,
                    Key::End => self.cursor_pos = self.text.len(),
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

/// Finds the byte index of the previous word boundary.
fn prev_word_boundary(text: &str, pos: usize) -> usize {
    let before = &text[..pos];
    let trimmed = before.trim_end();
    if trimmed.is_empty() {
        return 0;
    }
    trimmed
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0)
}

/// Finds the byte index of the next word boundary.
fn next_word_boundary(text: &str, pos: usize) -> usize {
    let after = &text[pos..];
    let skip_word = after
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(after.len());
    let rest = &after[skip_word..];
    let skip_space = rest
        .find(|c: char| c.is_alphanumeric() || c == '_')
        .unwrap_or(rest.len());
    pos + skip_word + skip_space
}
