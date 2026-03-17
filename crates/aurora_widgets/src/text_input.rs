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
/// Supports keyboard input, cursor navigation, text selection (Shift+arrows,
/// Shift+click, Ctrl+A), click-to-focus, and click-to-position-cursor.
/// Clicking outside the input unfocuses it.
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
///     .selection_color(Color::from_rgb(51, 120, 210))
///     .on_change(|text| println!("Input: {text}"))
/// ```
pub struct TextInput {
    id: u64,
    text: String,
    cursor_pos: usize,
    /// Byte index of the selection anchor, if a selection is active.
    /// The selected range is `min(anchor, cursor_pos)..max(anchor, cursor_pos)`.
    selection_anchor: Option<usize>,
    font: FontOptions,
    color: Color,
    placeholder_color: Color,
    selection_bg: Color,
    selection_fg: Color,
    background: Color,
    focused_background: Color,
    corners: Corners,
    padding: Edges,
    placeholder: String,
    focused: bool,
    width: Option<f32>,
    height: Option<f32>,
    text_layout: Option<TextLayout>,
    placeholder_layout: Option<TextLayout>,
    cursor_pixel_x: f32,
    /// Pixel x positions for each char boundary, used for click positioning and selection rendering.
    char_x_positions: Vec<f32>,
    on_change: Option<Box<dyn FnMut(&str)>>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            text: String::new(),
            cursor_pos: 0,
            selection_anchor: None,
            font: FontOptions::default(),
            color: Color::BLACK,
            placeholder_color: Color::new(160, 160, 160, 255),
            selection_bg: Color::new(51, 120, 210, 255),
            selection_fg: Color::WHITE,
            background: Color::new(245, 245, 245, 255),
            focused_background: Color::WHITE,
            corners: Corners::all(4.0),
            padding: Edges::symmetric(6.0, 10.0),
            placeholder: String::new(),
            focused: false,
            width: None,
            height: None,
            text_layout: None,
            placeholder_layout: None,
            cursor_pixel_x: 0.0,
            char_x_positions: Vec::new(),
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

    /// Sets the selection background color.
    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_bg = color;
        self
    }

    /// Sets the selected text foreground color.
    pub fn selection_text_color(mut self, color: Color) -> Self {
        self.selection_fg = color;
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

    /// Returns the selected byte range `(start, end)`, or `None` if no selection.
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            let lo = anchor.min(self.cursor_pos);
            let hi = anchor.max(self.cursor_pos);
            (lo, hi)
        })
    }

    /// Returns true if there is a non-empty selection.
    fn has_selection(&self) -> bool {
        self.selection_range()
            .is_some_and(|(lo, hi)| lo != hi)
    }

    /// Deletes the selected text and positions cursor at the start.
    fn delete_selection(&mut self) {
        if let Some((lo, hi)) = self.selection_range() {
            if lo != hi {
                self.text.drain(lo..hi);
                self.cursor_pos = lo;
                self.selection_anchor = None;
            }
        }
    }

    /// Clears any selection.
    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Starts or extends a selection from the current anchor.
    fn extend_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_pos);
        }
    }

    fn notify_change(&mut self) {
        if let Some(ref mut on_change) = self.on_change {
            on_change(&self.text);
        }
    }

    /// Returns the pixel x for a given byte position using the cached positions.
    fn pixel_x_for(&self, byte_pos: usize) -> f32 {
        // char_x_positions[i] = pixel x after the i-th char boundary
        // Index 0 = x after first char, etc.
        // We need to find the char index for this byte_pos.
        if byte_pos == 0 || self.char_x_positions.is_empty() {
            return 0.0;
        }
        let mut char_idx = 0;
        for (i, _) in self.text.char_indices() {
            if i >= byte_pos {
                break;
            }
            char_idx += 1;
        }
        if char_idx > 0 && char_idx <= self.char_x_positions.len() {
            self.char_x_positions[char_idx - 1]
        } else if char_idx == 0 {
            0.0
        } else {
            *self.char_x_positions.last().unwrap_or(&0.0)
        }
    }

    /// Finds the byte position closest to a pixel x offset from the text start.
    fn byte_pos_for_x(&self, x: f32) -> usize {
        if self.char_x_positions.is_empty() || x <= 0.0 {
            return 0;
        }
        let mut best_pos = 0;
        let mut best_dist = x.abs();
        let mut byte_idx_iter: Vec<usize> = self.text.char_indices().map(|(i, _)| i).collect();
        byte_idx_iter.push(self.text.len());
        // byte_idx_iter: [0, char1_start, char2_start, ..., text.len()]
        // char_x_positions: [x_after_char0, x_after_char1, ...]
        for (ci, &px) in self.char_x_positions.iter().enumerate() {
            let byte_pos = byte_idx_iter.get(ci + 1).copied().unwrap_or(self.text.len());
            let dist = (x - px).abs();
            if dist < best_dist {
                best_dist = dist;
                best_pos = byte_pos;
            }
        }
        best_pos
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

        // Build character x positions for cursor/selection positioning
        self.char_x_positions.clear();
        let mut running = String::new();
        for ch in self.text.chars() {
            running.push(ch);
            let cl = TextLayout::new(ctx.font_manager, &running, &resolved, self.color, None);
            self.char_x_positions.push(cl.size().width);
        }

        // Set cursor pixel x
        self.cursor_pixel_x = self.pixel_x_for(self.cursor_pos);

        Size::new(w, h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let bg = if self.focused {
            self.focused_background
        } else {
            self.background
        };
        canvas.fill_rounded_rect(rect, self.corners, bg);

        let text_x = rect.x1 + self.padding.left;
        let text_y = rect.y1 + self.padding.top;
        let font_size = self.font.effective_size();

        // Draw selection highlight
        if self.focused {
            if let Some((lo, hi)) = self.selection_range() {
                if lo != hi {
                    let sel_x0 = text_x + self.pixel_x_for(lo);
                    let sel_x1 = text_x + self.pixel_x_for(hi);
                    canvas.fill_rect(
                        Rect::new(sel_x0, text_y, sel_x1, text_y + font_size),
                        self.selection_bg,
                    );
                }
            }
        }

        // Draw text or placeholder
        if let Some(ref tl) = self.text_layout {
            canvas.draw_text(tl, text_x as i32, text_y as i32);
        } else if let Some(ref pl) = self.placeholder_layout {
            canvas.draw_text(pl, text_x as i32, text_y as i32);
        }

        // Draw cursor (only when focused and no selection, or at cursor end)
        if self.focused {
            let cursor_x = text_x + self.cursor_pixel_x;
            canvas.fill_rect(
                Rect::new(cursor_x, text_y, cursor_x + 1.5, text_y + font_size),
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

                    let click_x = click.position.x - rect.x1 - self.padding.left;
                    let new_pos = self.byte_pos_for_x(click_x);

                    // Shift+click extends selection
                    if click.state == MouseState::Pressed {
                        if self.selection_anchor.is_none() {
                            // No existing selection and no shift — just position
                            self.selection_anchor = None;
                        }
                    }

                    self.cursor_pos = new_pos;
                    self.cursor_pixel_x = self.pixel_x_for(new_pos);

                    return EventResponse {
                        handled: true,
                        request_focus: Some(self.id),
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                // Click outside — unfocus
                if click.state == MouseState::Pressed && !rect.contains(&click.position) {
                    self.focused = false;
                    self.clear_selection();
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
                // Replace selection if any
                if self.has_selection() {
                    self.delete_selection();
                }
                self.text.insert(self.cursor_pos, *ch);
                self.cursor_pos += ch.len_utf8();
                self.clear_selection();
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

                // Ctrl+A selects all
                if modifiers.ctrl && *key == Key::Character('a') {
                    self.selection_anchor = Some(0);
                    self.cursor_pos = self.text.len();
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }

                match key {
                    Key::Backspace => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else if self.cursor_pos > 0 {
                            let prev = self.text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            self.text.drain(prev..self.cursor_pos);
                            self.cursor_pos = prev;
                        }
                        self.clear_selection();
                        self.notify_change();
                    }
                    Key::Delete => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else if self.cursor_pos < self.text.len() {
                            let next = self.text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.text.len());
                            self.text.drain(self.cursor_pos..next);
                        }
                        self.clear_selection();
                        self.notify_change();
                    }
                    Key::Left => {
                        if modifiers.shift {
                            self.extend_selection();
                        } else if self.has_selection() {
                            let (lo, _) = self.selection_range().unwrap();
                            self.cursor_pos = lo;
                            self.clear_selection();
                            return EventResponse {
                                handled: true,
                                ..Default::default()
                            };
                        } else {
                            self.clear_selection();
                        }
                        let old = self.cursor_pos;
                        if modifiers.ctrl {
                            self.cursor_pos = prev_word_boundary(&self.text, self.cursor_pos);
                        } else if self.cursor_pos > 0 {
                            self.cursor_pos = self.text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                        }
                        if !modifiers.shift && old != self.cursor_pos {
                            self.clear_selection();
                        }
                    }
                    Key::Right => {
                        if modifiers.shift {
                            self.extend_selection();
                        } else if self.has_selection() {
                            let (_, hi) = self.selection_range().unwrap();
                            self.cursor_pos = hi;
                            self.clear_selection();
                            return EventResponse {
                                handled: true,
                                ..Default::default()
                            };
                        } else {
                            self.clear_selection();
                        }
                        let old = self.cursor_pos;
                        if modifiers.ctrl {
                            self.cursor_pos = next_word_boundary(&self.text, self.cursor_pos);
                        } else if self.cursor_pos < self.text.len() {
                            self.cursor_pos = self.text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.text.len());
                        }
                        if !modifiers.shift && old != self.cursor_pos {
                            self.clear_selection();
                        }
                    }
                    Key::Home => {
                        if modifiers.shift {
                            self.extend_selection();
                        } else {
                            self.clear_selection();
                        }
                        self.cursor_pos = 0;
                    }
                    Key::End => {
                        if modifiers.shift {
                            self.extend_selection();
                        } else {
                            self.clear_selection();
                        }
                        self.cursor_pos = self.text.len();
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
