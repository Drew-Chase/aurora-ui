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
use aurora_render::canvas::Canvas;
use aurora_text::text_layout::TextLayout;

use super::colors;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(2_000_000);

/// A text span with formatting flags.
#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl TextSpan {
    /// Creates a new plain text span.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italic: false,
            underline: false,
        }
    }

    /// Creates a new span with the given formatting flags.
    pub fn new(text: impl Into<String>, bold: bool, italic: bool, underline: bool) -> Self {
        Self {
            text: text.into(),
            bold,
            italic,
            underline,
        }
    }
}

/// A simplified rich text editor with a formatting toolbar.
///
/// Supports bold, italic, and underline formatting on text spans. The toolbar
/// provides toggle buttons for each format and keyboard shortcuts (Ctrl+B,
/// Ctrl+I, Ctrl+U).
///
/// # Example
/// ```ignore
/// RichTextEditor::new()
///     .placeholder("Write something...")
///     .height(250.0)
///     .width(500.0)
///     .on_change(|text| println!("plain text: {text}"))
/// ```
pub struct RichTextEditor {
    id: u64,
    spans: Vec<TextSpan>,
    toolbar: bool,
    placeholder: String,
    width: Option<f32>,
    height: f32,
    background: Color,
    border_color: Color,
    toolbar_bg: Color,
    corners: Corners,
    text_color: Color,
    font_size: f32,
    focused: bool,
    cursor_pos: usize,
    /// Selection anchor (reserved for future use).
    #[allow(dead_code)]
    selection_start: Option<usize>,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str)>>,
    content_layout: Option<TextLayout>,
    placeholder_layout: Option<TextLayout>,
    toolbar_layouts: Vec<Option<TextLayout>>,
    bold_active: bool,
    italic_active: bool,
    underline_active: bool,
    scroll_offset: f32,
    error: bool,
}

const TOOLBAR_HEIGHT: f32 = 36.0;
const TOOLBAR_BUTTON_SIZE: f32 = 28.0;
const TOOLBAR_BUTTON_SPACING: f32 = 4.0;
const TOOLBAR_PADDING: f32 = 4.0;
const CONTENT_PADDING: Edges = Edges {
    top: 8.0,
    right: 12.0,
    bottom: 8.0,
    left: 12.0,
};

impl RichTextEditor {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            spans: Vec::new(),
            toolbar: true,
            placeholder: "Type here...".to_string(),
            width: None,
            height: 200.0,
            background: colors::background(),
            border_color: colors::input_border(),
            toolbar_bg: colors::muted(),
            corners: Corners::all(6.0),
            text_color: colors::foreground(),
            font_size: 14.0,
            focused: false,
            cursor_pos: 0,
            selection_start: None,
            on_change: None,
            content_layout: None,
            placeholder_layout: None,
            toolbar_layouts: Vec::new(),
            bold_active: false,
            italic_active: false,
            underline_active: false,
            scroll_offset: 0.0,
            error: false,
        }
    }

    /// Sets initial text spans.
    pub fn spans(mut self, spans: Vec<TextSpan>) -> Self {
        self.cursor_pos = spans.iter().map(|s| s.text.len()).sum();
        self.spans = spans;
        self
    }

    /// Shows or hides the formatting toolbar.
    pub fn toolbar(mut self, show: bool) -> Self {
        self.toolbar = show;
        self
    }

    /// Sets the placeholder text shown when empty.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Sets a fixed width.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Sets the height (includes toolbar).
    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Sets the border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Sets the toolbar background color.
    pub fn toolbar_bg(mut self, color: Color) -> Self {
        self.toolbar_bg = color;
        self
    }

    /// Sets the corner radii.
    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    /// Sets the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Sets the font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Registers a callback fired when content changes, receiving the plain text.
    pub fn on_change(mut self, cb: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    /// Enables or disables the error state (red border).
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Returns the plain text by concatenating all span texts.
    pub fn plain_text(&self) -> String {
        let mut result = String::new();
        for span in &self.spans {
            result.push_str(&span.text);
        }
        result
    }

    /// Returns whether the editor is empty (no text in any span).
    fn is_empty(&self) -> bool {
        self.spans.iter().all(|s| s.text.is_empty())
    }

    fn notify_change(&mut self) {
        if self.on_change.is_some() {
            let text = self.plain_text();
            if let Some(ref mut cb) = self.on_change {
                cb(&text);
            }
        }
    }

    /// Inserts a character at the current cursor position with the current format state.
    fn insert_char(&mut self, ch: char) {
        let plain = self.plain_text();
        let byte_pos = self.cursor_pos.min(plain.len());

        // Find which span and offset the cursor is in.
        let mut offset = 0;
        let mut target_span = self.spans.len(); // append
        let mut target_offset = 0;
        for (i, span) in self.spans.iter().enumerate() {
            let span_end = offset + span.text.len();
            if byte_pos <= span_end {
                target_span = i;
                target_offset = byte_pos - offset;
                break;
            }
            offset = span_end;
        }

        let current_bold = self.bold_active;
        let current_italic = self.italic_active;
        let current_underline = self.underline_active;

        if target_span < self.spans.len() {
            let span = &self.spans[target_span];
            // If the span has the same formatting, insert into it directly.
            if span.bold == current_bold
                && span.italic == current_italic
                && span.underline == current_underline
            {
                self.spans[target_span].text.insert(target_offset, ch);
            } else {
                // Need to split the span and insert a new one with current formatting.
                let old_text = self.spans[target_span].text.clone();
                let (before, after) = old_text.split_at(target_offset);
                let old_bold = self.spans[target_span].bold;
                let old_italic = self.spans[target_span].italic;
                let old_underline = self.spans[target_span].underline;

                let mut new_spans = Vec::new();
                if !before.is_empty() {
                    new_spans.push(TextSpan::new(before, old_bold, old_italic, old_underline));
                }
                new_spans.push(TextSpan::new(
                    ch.to_string(),
                    current_bold,
                    current_italic,
                    current_underline,
                ));
                if !after.is_empty() {
                    new_spans.push(TextSpan::new(after, old_bold, old_italic, old_underline));
                }

                self.spans.splice(target_span..=target_span, new_spans);
            }
        } else {
            // Append a new span.
            self.spans.push(TextSpan::new(
                ch.to_string(),
                current_bold,
                current_italic,
                current_underline,
            ));
        }

        self.cursor_pos += ch.len_utf8();
        self.merge_adjacent_spans();
    }

    /// Deletes the character before the cursor (backspace).
    fn delete_before_cursor(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        let plain = self.plain_text();
        // Find the previous character boundary.
        let prev = plain[..self.cursor_pos]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);

        self.delete_range(prev, self.cursor_pos);
        self.cursor_pos = prev;
    }

    /// Deletes the character after the cursor (delete key).
    fn delete_after_cursor(&mut self) {
        let plain = self.plain_text();
        if self.cursor_pos >= plain.len() {
            return;
        }

        let next = plain[self.cursor_pos..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.cursor_pos + i)
            .unwrap_or(plain.len());

        self.delete_range(self.cursor_pos, next);
    }

    /// Deletes the byte range [start, end) across spans.
    fn delete_range(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let mut offset = 0;
        let mut new_spans = Vec::new();
        for span in &self.spans {
            let span_start = offset;
            let span_end = offset + span.text.len();
            offset = span_end;

            if span_end <= start || span_start >= end {
                // Entirely outside the deletion range — keep.
                new_spans.push(span.clone());
            } else {
                // Partially or fully inside the deletion range.
                let mut text = String::new();
                if span_start < start {
                    text.push_str(&span.text[..(start - span_start)]);
                }
                if span_end > end {
                    text.push_str(&span.text[(end - span_start)..]);
                }
                if !text.is_empty() {
                    new_spans.push(TextSpan::new(
                        text,
                        span.bold,
                        span.italic,
                        span.underline,
                    ));
                }
            }
        }
        self.spans = new_spans;
        self.merge_adjacent_spans();
    }

    /// Merges adjacent spans with identical formatting.
    fn merge_adjacent_spans(&mut self) {
        if self.spans.len() < 2 {
            return;
        }
        let mut merged = vec![self.spans[0].clone()];
        for span in &self.spans[1..] {
            let last = merged.last().unwrap();
            if last.bold == span.bold
                && last.italic == span.italic
                && last.underline == span.underline
            {
                let len = merged.len();
                merged[len - 1].text.push_str(&span.text);
            } else if !span.text.is_empty() {
                merged.push(span.clone());
            }
        }
        // Remove empty spans.
        merged.retain(|s| !s.text.is_empty());
        self.spans = merged;
    }

    /// Moves the cursor left by one character.
    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            let plain = self.plain_text();
            self.cursor_pos = plain[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Moves the cursor right by one character.
    fn move_cursor_right(&mut self) {
        let plain = self.plain_text();
        if self.cursor_pos < plain.len() {
            self.cursor_pos = plain[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(plain.len());
        }
    }

    /// Returns the rect for a toolbar button at the given index.
    fn toolbar_button_rect(&self, rect: &Rect, index: usize) -> Rect {
        let x = rect.x1
            + TOOLBAR_PADDING
            + index as f32 * (TOOLBAR_BUTTON_SIZE + TOOLBAR_BUTTON_SPACING);
        let y = rect.y1 + (TOOLBAR_HEIGHT - TOOLBAR_BUTTON_SIZE) / 2.0;
        Rect::new(x, y, x + TOOLBAR_BUTTON_SIZE, y + TOOLBAR_BUTTON_SIZE)
    }

    /// Returns the content area rect (below toolbar).
    fn content_rect(&self, rect: &Rect) -> Rect {
        let top = if self.toolbar {
            rect.y1 + TOOLBAR_HEIGHT
        } else {
            rect.y1
        };
        Rect::new(rect.x1, top, rect.x2, rect.y2)
    }

    /// Returns the toolbar area rect.
    fn toolbar_rect(&self, rect: &Rect) -> Rect {
        Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + TOOLBAR_HEIGHT)
    }

    /// Returns the index (0=B, 1=I, 2=U) of a clicked toolbar button, if any.
    fn hit_toolbar_button(&self, rect: &Rect, x: f32, y: f32) -> Option<usize> {
        if !self.toolbar {
            return None;
        }
        let point = aurora_core::geometry::point::Point::new(x, y);
        (0..3).find(|&i| self.toolbar_button_rect(rect, i).contains(&point))
    }
}

impl Default for RichTextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for RichTextEditor {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let inner_w =
            (w - CONTENT_PADDING.left - CONTENT_PADDING.right).max(0.0);

        // Content text layout
        let plain = self.plain_text();
        if !plain.is_empty() {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(self.font_size);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = TextLayout::new(
                ctx.font_manager,
                &plain,
                &opts,
                self.text_color,
                None,
            );
            tl.set_max_width(ctx.font_manager, inner_w);
            self.content_layout = Some(tl);
        } else {
            self.content_layout = None;
        }

        // Placeholder layout
        if !self.placeholder.is_empty() && self.is_empty() {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(self.font_size);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = TextLayout::new(
                ctx.font_manager,
                &self.placeholder,
                &opts,
                colors::muted_foreground(),
                None,
            );
            tl.set_max_width(ctx.font_manager, inner_w);
            self.placeholder_layout = Some(tl);
        } else {
            self.placeholder_layout = None;
        }

        // Toolbar button label layouts (B, I, U)
        if self.toolbar {
            let labels = ["B", "I", "U"];
            self.toolbar_layouts.clear();
            for label in &labels {
                let mut opts = ctx.font_options.clone();
                opts.size = Some(13.0);
                opts.weight = Some(aurora_text::font_options::FontWeight::Bold);
                let tl = TextLayout::new(
                    ctx.font_manager,
                    label,
                    &opts,
                    colors::foreground(),
                    None,
                );
                self.toolbar_layouts.push(Some(tl));
            }
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background
        canvas.fill_rounded_rect(rect, self.corners, self.background);

        // Border
        let border_color = if self.error {
            colors::destructive()
        } else if self.focused {
            colors::ring()
        } else {
            self.border_color
        };
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Toolbar
        if self.toolbar {
            let tr = self.toolbar_rect(&rect);
            canvas.fill_rect(
                Rect::new(tr.x1 + 1.0, tr.y1 + 1.0, tr.x2 - 1.0, tr.y2),
                self.toolbar_bg,
            );
            // Separator line below toolbar
            canvas.fill_rect(
                Rect::new(tr.x1, tr.y2 - 1.0, tr.x2, tr.y2),
                self.border_color,
            );

            // Toolbar buttons
            let active_flags = [self.bold_active, self.italic_active, self.underline_active];
            for (i, &is_active) in active_flags.iter().enumerate() {
                let btn_rect = self.toolbar_button_rect(&rect, i);
                let btn_corners = Corners::all(4.0);

                if is_active {
                    canvas.fill_rounded_rect(btn_rect, btn_corners, colors::primary());
                }

                // Draw label centered
                if let Some(Some(tl)) = self.toolbar_layouts.get(i) {
                    let ts = tl.size();
                    let x = btn_rect.x1 + (btn_rect.width() - ts.width) / 2.0;
                    let y = btn_rect.y1 + (btn_rect.height() - ts.height) / 2.0;
                    canvas.draw_text(tl, x as i32, y as i32);
                }
            }
        }

        // Content area
        let content = self.content_rect(&rect);
        let text_rect = Rect::new(
            content.x1 + CONTENT_PADDING.left,
            content.y1 + CONTENT_PADDING.top,
            content.x2 - CONTENT_PADDING.right,
            content.y2 - CONTENT_PADDING.bottom,
        );
        canvas.push_clip(text_rect);

        let tx = text_rect.x1;
        let ty = text_rect.y1 - self.scroll_offset;

        // Draw content or placeholder
        if let Some(ref tl) = self.content_layout {
            canvas.draw_text(tl, tx as i32, ty as i32);
        } else if !self.focused
            && let Some(ref pl) = self.placeholder_layout
        {
            canvas.draw_text(pl, tx as i32, ty as i32);
        }

        // Cursor
        if self.focused {
            let cursor_x = tx;
            let cursor_y = ty;
            let line_height = self.font_size * 1.4;
            canvas.fill_rect(
                Rect::new(
                    cursor_x,
                    cursor_y,
                    cursor_x + 1.5,
                    cursor_y + line_height,
                ),
                self.text_color,
            );
        }

        canvas.pop_clip();
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn tab_index(&self) -> Option<u32> {
        None
    }

    fn widget_id(&self) -> Option<u64> {
        Some(self.id)
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
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

                    // Check toolbar button clicks.
                    if let Some(btn) = self.hit_toolbar_button(&rect, e.position.x, e.position.y) {
                        match btn {
                            0 => self.bold_active = !self.bold_active,
                            1 => self.italic_active = !self.italic_active,
                            2 => self.underline_active = !self.underline_active,
                            _ => {}
                        }
                        return EventResponse {
                            status: EventStatus::Consumed,
                            request_focus: Some(self.id),
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }

                    // Click in content area positions cursor (simplified: go to end).
                    let content = self.content_rect(&rect);
                    if content.contains(&e.position) {
                        // For simplicity, clicking puts cursor at end.
                        self.cursor_pos = self.plain_text().len();
                    }

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
                    // Check if hovering toolbar buttons.
                    if self.toolbar {
                        let tr = self.toolbar_rect(&rect);
                        if tr.contains(pos) {
                            return EventResponse {
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                    }
                    return EventResponse {
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(scroll)) => {
                if !self.focused {
                    return EventResponse::default();
                }
                self.scroll_offset = (self.scroll_offset - scroll * 20.0).max(0.0);
                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !self.focused {
                    return EventResponse::default();
                }
                self.insert_char(*ch);
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

                // Ctrl+B — toggle bold
                if modifiers.ctrl && *key == Key::Character('b') {
                    self.bold_active = !self.bold_active;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                // Ctrl+I — toggle italic
                if modifiers.ctrl && *key == Key::Character('i') {
                    self.italic_active = !self.italic_active;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                // Ctrl+U — toggle underline
                if modifiers.ctrl && *key == Key::Character('u') {
                    self.underline_active = !self.underline_active;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                match key {
                    Key::Enter => {
                        self.insert_char('\n');
                        self.notify_change();
                    }
                    Key::Backspace => {
                        self.delete_before_cursor();
                        self.notify_change();
                    }
                    Key::Delete => {
                        self.delete_after_cursor();
                        self.notify_change();
                    }
                    Key::Left => {
                        self.move_cursor_left();
                    }
                    Key::Right => {
                        self.move_cursor_right();
                    }
                    Key::Home => {
                        self.cursor_pos = 0;
                    }
                    Key::End => {
                        self.cursor_pos = self.plain_text().len();
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
        let mut info =
            aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::MultilineTextInput);
        if !self.placeholder.is_empty() {
            info = info.with_label(self.placeholder.clone());
        }
        let text = self.plain_text();
        if !text.is_empty() {
            info = info.with_value(text);
        }
        info.multiline = true;
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let editor = RichTextEditor::new();
        assert_eq!(editor.placeholder, "Type here...");
        assert_eq!(editor.height, 200.0);
        assert!(editor.toolbar);
        assert!(!editor.bold_active);
        assert!(!editor.italic_active);
        assert!(!editor.underline_active);
        assert_eq!(editor.font_size, 14.0);
        assert_eq!(editor.cursor_pos, 0);
        assert!(editor.selection_start.is_none());
        assert!(!editor.focused);
        assert!(!editor.error);
        assert!(editor.spans.is_empty());
    }

    #[test]
    fn plain_text_extraction() {
        let editor = RichTextEditor::new().spans(vec![
            TextSpan::plain("Hello "),
            TextSpan::new("world", true, false, false),
            TextSpan::plain("!"),
        ]);
        assert_eq!(editor.plain_text(), "Hello world!");
    }

    #[test]
    fn empty_placeholder() {
        let editor = RichTextEditor::new();
        assert!(editor.is_empty());
        assert_eq!(editor.plain_text(), "");

        let editor = RichTextEditor::new().spans(vec![TextSpan::plain("text")]);
        assert!(!editor.is_empty());
    }

    #[test]
    fn span_insertion() {
        let mut editor = RichTextEditor::new();
        editor.bold_active = true;
        editor.insert_char('A');
        assert_eq!(editor.spans.len(), 1);
        assert!(editor.spans[0].bold);
        assert_eq!(editor.spans[0].text, "A");

        editor.bold_active = false;
        editor.insert_char('B');
        assert_eq!(editor.spans.len(), 2);
        assert!(!editor.spans[1].bold);
        assert_eq!(editor.plain_text(), "AB");
    }

    #[test]
    fn format_toggle() {
        let mut editor = RichTextEditor::new();
        assert!(!editor.bold_active);
        editor.bold_active = !editor.bold_active;
        assert!(editor.bold_active);

        assert!(!editor.italic_active);
        editor.italic_active = !editor.italic_active;
        assert!(editor.italic_active);

        assert!(!editor.underline_active);
        editor.underline_active = !editor.underline_active;
        assert!(editor.underline_active);
    }

    #[test]
    fn cursor_movement() {
        let mut editor = RichTextEditor::new().spans(vec![TextSpan::plain("Hello")]);

        // Cursor should be at end after setting spans.
        assert_eq!(editor.cursor_pos, 5);

        editor.move_cursor_left();
        assert_eq!(editor.cursor_pos, 4);

        editor.move_cursor_right();
        assert_eq!(editor.cursor_pos, 5);

        // Moving right at end should stay at end.
        editor.move_cursor_right();
        assert_eq!(editor.cursor_pos, 5);

        // Move all the way to the start.
        for _ in 0..10 {
            editor.move_cursor_left();
        }
        assert_eq!(editor.cursor_pos, 0);

        // Moving left at start should stay at start.
        editor.move_cursor_left();
        assert_eq!(editor.cursor_pos, 0);
    }

    #[test]
    fn backspace_deletes_character() {
        let mut editor = RichTextEditor::new().spans(vec![TextSpan::plain("ABC")]);
        assert_eq!(editor.cursor_pos, 3);

        editor.delete_before_cursor();
        assert_eq!(editor.plain_text(), "AB");
        assert_eq!(editor.cursor_pos, 2);

        editor.delete_before_cursor();
        assert_eq!(editor.plain_text(), "A");
        assert_eq!(editor.cursor_pos, 1);
    }

    #[test]
    fn delete_after_cursor_works() {
        let mut editor = RichTextEditor::new().spans(vec![TextSpan::plain("ABC")]);
        editor.cursor_pos = 0;

        editor.delete_after_cursor();
        assert_eq!(editor.plain_text(), "BC");
        assert_eq!(editor.cursor_pos, 0);
    }

    #[test]
    fn merge_adjacent_spans() {
        let mut editor = RichTextEditor::new().spans(vec![
            TextSpan::plain("A"),
            TextSpan::plain("B"),
            TextSpan::plain("C"),
        ]);
        editor.merge_adjacent_spans();
        assert_eq!(editor.spans.len(), 1);
        assert_eq!(editor.spans[0].text, "ABC");
    }
}
