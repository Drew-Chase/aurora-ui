use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent, Modifiers};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;
use aurora_text::font_options::FontOptions;
use aurora_text::text_layout::TextLayout;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Callback for key events. Receives the key and modifiers.
pub type OnKeyCallback = Box<dyn FnMut(&Key, &Modifiers)>;
/// Callback for text change events. Receives the current text value.
pub type OnChangeCallback = Box<dyn FnMut(&str)>;

/// A single-line text input field.
///
/// Supports keyboard input, cursor navigation, text selection (Shift+arrows,
/// Shift+click, Ctrl+A), click-to-focus, click-to-position-cursor, password
/// masking, Tab navigation, and Enter to submit.
///
/// # Tab Navigation
///
/// Set a `tab_index` on each focusable widget. When the user presses Tab,
/// focus moves to the next widget by tab index. Shift+Tab moves backward.
/// Tab indices are global — the framework collects them from all widgets
/// in the tree and cycles through them in order.
///
/// # Example
///
/// ```no_run
/// use aurora_ui::prelude::*;
///
/// TextInput::new()
///     .placeholder("Username")
///     .tab_index(1)
///     .on_submit(|| println!("submitted!"))
///     .on_change(|text| println!("Input: {text}"))
/// ```
pub struct TextInput {
    id: u64,
    text: String,
    cursor_pos: usize,
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
    password: bool,
    tab_index: Option<u32>,
    width: Option<f32>,
    height: Option<f32>,
    text_layout: Option<TextLayout>,
    placeholder_layout: Option<TextLayout>,
    cursor_pixel_x: f32,
    scroll_offset: f32,
    char_x_positions: Vec<f32>,
    on_change: Option<OnChangeCallback>,
    on_submit: Option<Box<dyn FnMut()>>,
    on_key_down: Option<OnKeyCallback>,
    on_key_up: Option<OnKeyCallback>,
    mouse_down: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            text: String::new(),
            cursor_pos: 0,
            selection_anchor: None,
            font: FontOptions::default(),
            color: aurora_theme::color(aurora_theme::slots::FOREGROUND),
            placeholder_color: aurora_theme::color(aurora_theme::slots::MUTED_FOREGROUND),
            selection_bg: aurora_theme::color(aurora_theme::slots::PRIMARY),
            selection_fg: aurora_theme::color(aurora_theme::slots::PRIMARY_FOREGROUND),
            background: aurora_theme::color(aurora_theme::slots::MUTED),
            focused_background: aurora_theme::color(aurora_theme::slots::BACKGROUND),
            corners: Corners::all(4.0),
            padding: Edges::symmetric(6.0, 10.0),
            placeholder: String::new(),
            focused: false,
            password: false,
            tab_index: None,
            width: None,
            height: None,
            text_layout: None,
            placeholder_layout: None,
            cursor_pixel_x: 0.0,
            scroll_offset: 0.0,
            char_x_positions: Vec::new(),
            mouse_down: false,
            on_change: None,
            on_submit: None,
            on_key_down: None,
            on_key_up: None,
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

    /// Enables password mode — text is displayed as dots.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Sets a stable widget ID that persists across Composite rebuilds.
    ///
    /// Use this when the TextInput is inside a Composite whose state changes
    /// on every keystroke — it allows the framework to restore focus after
    /// the widget tree is rebuilt.
    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    /// Sets the tab index for keyboard navigation.
    ///
    /// When Tab is pressed, focus moves to the widget with the next higher
    /// tab index. Shift+Tab moves to the previous.
    pub fn tab_index(mut self, index: u32) -> Self {
        self.tab_index = Some(index);
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

    /// Registers a callback invoked when Enter is pressed while focused.
    pub fn on_submit(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on key press while focused.
    pub fn on_key_down(mut self, f: impl FnMut(&Key, &Modifiers) + 'static) -> Self {
        self.on_key_down = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on key release while focused.
    pub fn on_key_up(mut self, f: impl FnMut(&Key, &Modifiers) + 'static) -> Self {
        self.on_key_up = Some(Box::new(f));
        self
    }

    /// Returns the display text — masked with dots if in password mode.
    fn display_text(&self) -> String {
        if self.password {
            "\u{2022}".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            let lo = anchor.min(self.cursor_pos);
            let hi = anchor.max(self.cursor_pos);
            (lo, hi)
        })
    }

    fn has_selection(&self) -> bool {
        self.selection_range()
            .is_some_and(|(lo, hi)| lo != hi)
    }

    fn delete_selection(&mut self) {
        if let Some((lo, hi)) = self.selection_range()
            && lo != hi
        {
            self.text.drain(lo..hi);
            self.cursor_pos = lo;
            self.selection_anchor = None;
        }
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

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

    /// Maps a byte position in the real text to a byte position in the display text.
    fn display_byte_pos(&self, byte_pos: usize) -> usize {
        if !self.password {
            return byte_pos;
        }
        // In password mode, each char becomes a bullet (3 bytes in UTF-8).
        let char_count = self.text[..byte_pos].chars().count();
        char_count * "\u{2022}".len()
    }

    fn pixel_x_for(&self, byte_pos: usize) -> f32 {
        let display_pos = self.display_byte_pos(byte_pos);
        if display_pos == 0 || self.char_x_positions.is_empty() {
            return 0.0;
        }
        let display_text = self.display_text();
        let mut char_idx = 0;
        for (i, _) in display_text.char_indices() {
            if i >= display_pos {
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

    fn byte_pos_for_x(&self, x: f32) -> usize {
        if self.char_x_positions.is_empty() || x <= 0.0 {
            return 0;
        }
        let display_text = self.display_text();
        let mut byte_indices: Vec<usize> = display_text.char_indices().map(|(i, _)| i).collect();
        byte_indices.push(display_text.len());

        // Find closest char boundary in display text
        let mut best_display_pos = 0;
        let mut best_dist = x.abs();
        for (ci, &px) in self.char_x_positions.iter().enumerate() {
            let dist = (x - px).abs();
            if dist < best_dist {
                best_dist = dist;
                best_display_pos = byte_indices.get(ci + 1).copied().unwrap_or(display_text.len());
            }
        }

        // Map display byte pos back to real byte pos
        if !self.password {
            return best_display_pos;
        }
        // In password mode, each display char = 1 real char
        let display_char_count = display_text[..best_display_pos].chars().count();
        self.text
            .char_indices()
            .nth(display_char_count)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Sets focus state and returns whether focus changed.
    pub fn set_focused(&mut self, focused: bool) {
        if !focused {
            self.clear_selection();
        }
        self.focused = focused;
    }

    /// Returns this widget's tab index, if set.
    pub fn get_tab_index(&self) -> Option<u32> {
        self.tab_index
    }

    /// Returns this widget's unique ID.
    pub fn get_id(&self) -> u64 {
        self.id
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

        let display = self.display_text();

        let content_width = max_width;

        // Build text layout from display text — no max_width so text stays on one line
        if !display.is_empty() {
            let tl =
                TextLayout::new(ctx.font_manager, &display, &resolved, self.color, None);
            self.text_layout = Some(tl);
        } else {
            self.text_layout = None;
        }

        // Build placeholder layout — constrained to widget width
        if !self.placeholder.is_empty() && self.text.is_empty() {
            let mut pl = TextLayout::new(
                ctx.font_manager,
                &self.placeholder,
                &resolved,
                self.placeholder_color,
                None,
            );
            pl.set_max_width(ctx.font_manager, content_width);
            self.placeholder_layout = Some(pl);
        } else {
            self.placeholder_layout = None;
        }

        // Build character x positions from glyph advances (O(n) vs prior O(n²))
        self.char_x_positions = if let Some(ref tl) = self.text_layout {
            tl.char_x_positions()
        } else {
            Vec::new()
        };

        self.cursor_pixel_x = self.pixel_x_for(self.cursor_pos);

        // Scroll to keep cursor visible within the content area
        if self.cursor_pixel_x - self.scroll_offset > content_width {
            self.scroll_offset = self.cursor_pixel_x - content_width;
        }
        if self.cursor_pixel_x - self.scroll_offset < 0.0 {
            self.scroll_offset = self.cursor_pixel_x;
        }
        if self.scroll_offset < 0.0 {
            self.scroll_offset = 0.0;
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

        let content_rect = Rect::new(
            rect.x1 + self.padding.left,
            rect.y1 + self.padding.top,
            rect.x2 - self.padding.right,
            rect.y2 - self.padding.bottom,
        );
        canvas.push_clip(content_rect);

        let text_x = rect.x1 + self.padding.left - self.scroll_offset;
        let text_y = rect.y1 + self.padding.top;
        let font_size = self.font.effective_size();

        // Draw selection highlight
        if self.focused
            && let Some((lo, hi)) = self.selection_range()
            && lo != hi
        {
            let sel_x0 = text_x + self.pixel_x_for(lo);
            let sel_x1 = text_x + self.pixel_x_for(hi);
            canvas.fill_rect(
                Rect::new(sel_x0, text_y, sel_x1, text_y + font_size),
                self.selection_bg,
            );
        }

        // Draw text or placeholder
        if let Some(ref tl) = self.text_layout {
            canvas.draw_text(tl, text_x as i32, text_y as i32);
        } else if let Some(ref pl) = self.placeholder_layout {
            canvas.draw_text(pl, (rect.x1 + self.padding.left) as i32, text_y as i32);
        }

        // Draw cursor
        if self.focused {
            let cursor_x = text_x + self.cursor_pixel_x;
            canvas.fill_rect(
                Rect::new(cursor_x, text_y, cursor_x + 1.5, text_y + font_size),
                self.color,
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
            WidgetEvent::Focus(target_id, select_all) => {
                if *target_id == self.id {
                    self.focused = true;
                    if *select_all && !self.text.is_empty() {
                        self.cursor_pos = self.text.len();
                        self.selection_anchor = Some(0);
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                } else if self.focused {
                    // A different widget is being focused — clear our focus
                    self.focused = false;
                    self.mouse_down = false;
                    self.clear_selection();
                }
                EventResponse::default()
            }
            WidgetEvent::Blur(target_id) => {
                if *target_id == self.id {
                    self.focused = false;
                    self.mouse_down = false;
                    self.clear_selection();
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(click)) => {
                if click.state == MouseState::Pressed && rect.contains(&click.position) {
                    self.focused = true;
                    self.mouse_down = true;

                    let click_x = click.position.x - rect.x1 - self.padding.left + self.scroll_offset;
                    let new_pos = self.byte_pos_for_x(click_x);
                    self.cursor_pos = new_pos;
                    self.clear_selection();
                    self.cursor_pixel_x = self.pixel_x_for(new_pos);

                    return EventResponse {
                        handled: true,
                        request_focus: Some(self.id),
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                if click.state == MouseState::Released {
                    self.mouse_down = false;
                }
                if click.state == MouseState::Pressed && !rect.contains(&click.position) {
                    self.focused = false;
                    self.mouse_down = false;
                    self.clear_selection();
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.mouse_down && self.focused {
                    let drag_x = pos.x - rect.x1 - self.padding.left + self.scroll_offset;
                    let new_pos = self.byte_pos_for_x(drag_x);
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor_pos);
                    }
                    self.cursor_pos = new_pos;
                    self.cursor_pixel_x = self.pixel_x_for(new_pos);
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
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

                // Fire on_key_down callback
                if let Some(ref mut cb) = self.on_key_down {
                    cb(key, modifiers);
                }

                // Tab navigation
                if *key == Key::Tab {
                    self.focused = false;
                    self.clear_selection();
                    return EventResponse {
                        handled: true,
                        request_focus: Some(self.id),
                        focus_next: !modifiers.shift,
                        focus_prev: modifiers.shift,
                        ..Default::default()
                    };
                }

                // Enter submits
                if *key == Key::Enter {
                    if let Some(ref mut on_submit) = self.on_submit {
                        on_submit();
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
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

                // Ctrl+C — copy selection to clipboard
                if modifiers.ctrl && *key == Key::Character('c') {
                    #[cfg(feature = "text")]
                    if let Some((lo, hi)) = self.selection_range() {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(&self.text[lo..hi]);
                        }
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }

                // Ctrl+X — cut selection to clipboard
                if modifiers.ctrl && *key == Key::Character('x') {
                    #[cfg(feature = "text")]
                    if let Some((lo, hi)) = self.selection_range() {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(&self.text[lo..hi]);
                        }
                    }
                    if self.has_selection() {
                        self.delete_selection();
                        self.clear_selection();
                        self.notify_change();
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }

                // Ctrl+V — paste from clipboard
                if modifiers.ctrl && *key == Key::Character('v') {
                    #[cfg(feature = "text")]
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        if let Ok(text) = cb.get_text() {
                            if self.has_selection() {
                                self.delete_selection();
                            }
                            self.text.insert_str(self.cursor_pos, &text);
                            self.cursor_pos += text.len();
                            self.clear_selection();
                            self.notify_change();
                        }
                    }
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }

                match key {
                    Key::Backspace => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else if modifiers.ctrl {
                            let boundary = prev_word_boundary(&self.text, self.cursor_pos);
                            self.text.drain(boundary..self.cursor_pos);
                            self.cursor_pos = boundary;
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
                        } else if modifiers.ctrl {
                            let boundary = next_word_boundary(&self.text, self.cursor_pos);
                            self.text.drain(self.cursor_pos..boundary);
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
                        if modifiers.ctrl {
                            self.cursor_pos = prev_word_boundary(&self.text, self.cursor_pos);
                        } else if self.cursor_pos > 0 {
                            self.cursor_pos = self.text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                        }
                        if !modifiers.shift {
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
                        if modifiers.ctrl {
                            self.cursor_pos = next_word_boundary(&self.text, self.cursor_pos);
                        } else if self.cursor_pos < self.text.len() {
                            self.cursor_pos = self.text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.text.len());
                        }
                        if !modifiers.shift {
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
            WidgetEvent::Keyboard(KeyboardEvent::KeyReleased { key, modifiers }) => {
                if !self.focused {
                    return EventResponse::default();
                }
                if let Some(ref mut cb) = self.on_key_up {
                    cb(key, modifiers);
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}

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
