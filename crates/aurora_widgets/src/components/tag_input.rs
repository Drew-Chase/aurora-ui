use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// A multi-value input with removable tags.
///
/// Users type text and press Enter or comma to create a tag. Tags are
/// displayed as rounded pills with an optional X button for removal.
/// Backspace on an empty input removes the last tag.
///
/// # Example
/// ```ignore
/// TagInput::new()
///     .tags(vec!["Rust", "Aurora", "UI"])
///     .placeholder("Add tag...")
///     .max_tags(10)
///     .width(400.0)
///     .on_change(|tags| println!("{tags:?}"))
/// ```
pub struct TagInput {
    tags: Vec<String>,
    input_text: String,
    placeholder: String,
    max_tags: Option<usize>,
    removable: bool,
    width: Option<f32>,
    min_height: f32,
    tag_height: f32,
    tag_spacing: f32,
    tag_color: Color,
    tag_text_color: Color,
    background: Color,
    border_color: Color,
    corners: Corners,
    padding: Edges,
    focused: bool,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&[String])>>,
    tag_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    input_layout: Option<aurora_text::text_layout::TextLayout>,
    placeholder_layout: Option<aurora_text::text_layout::TextLayout>,
    hover_remove: Option<usize>,
    tag_rects: Vec<Rect>,
    error: bool,
    #[allow(dead_code)]
    text_color: Color,
    cursor_pos: usize,
    selection_start: Option<usize>,
    mouse_down: bool,
}

impl TagInput {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            input_text: String::new(),
            placeholder: "Add tag...".to_string(),
            max_tags: None,
            removable: true,
            width: None,
            min_height: 40.0,
            tag_height: 26.0,
            tag_spacing: 6.0,
            tag_color: colors::primary(),
            tag_text_color: colors::primary_foreground(),
            background: colors::background(),
            border_color: colors::input_border(),
            corners: Corners::all(6.0),
            padding: Edges::new(6.0, 8.0, 6.0, 8.0),
            focused: false,
            on_change: None,
            tag_layouts: Vec::new(),
            input_layout: None,
            placeholder_layout: None,
            hover_remove: None,
            tag_rects: Vec::new(),
            error: false,
            text_color: colors::foreground(),
            cursor_pos: 0,
            selection_start: None,
            mouse_down: false,
        }
    }

    pub fn tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    pub fn input_text(mut self, text: impl Into<String>) -> Self {
        self.input_text = text.into();
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn max_tags(mut self, max: usize) -> Self {
        self.max_tags = Some(max);
        self
    }

    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }

    pub fn tag_height(mut self, height: f32) -> Self {
        self.tag_height = height;
        self
    }

    pub fn tag_spacing(mut self, spacing: f32) -> Self {
        self.tag_spacing = spacing;
        self
    }

    pub fn tag_color(mut self, color: Color) -> Self {
        self.tag_color = color;
        self
    }

    pub fn tag_text_color(mut self, color: Color) -> Self {
        self.tag_text_color = color;
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

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(&[String]) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    /// Try to add a tag. Returns true if the tag was added.
    fn try_add_tag(&mut self, tag: String) -> bool {
        let trimmed = tag.trim().to_string();
        if trimmed.is_empty() {
            return false;
        }
        // Duplicate check (case-insensitive)
        if self.tags.iter().any(|t| t.eq_ignore_ascii_case(&trimmed)) {
            return false;
        }
        // Max tags check
        if let Some(max) = self.max_tags
            && self.tags.len() >= max
        {
            return false;
        }
        self.tags.push(trimmed);
        true
    }

    /// Remove a tag by index. Returns true if a tag was removed.
    fn remove_tag(&mut self, index: usize) -> bool {
        if index < self.tags.len() {
            self.tags.remove(index);
            true
        } else {
            false
        }
    }

    fn notify_change(&mut self) {
        if let Some(ref mut cb) = self.on_change {
            cb(&self.tags);
        }
    }

    /// Compute the X button rect for a tag at the given rect position.
    fn remove_button_rect(&self, tag_rect: &Rect) -> Rect {
        let btn_size = 16.0;
        let btn_x = tag_rect.x2 - 8.0 - btn_size;
        let btn_y = tag_rect.y1 + (self.tag_height - btn_size) / 2.0;
        Rect::new(btn_x, btn_y, btn_x + btn_size, btn_y + btn_size)
    }

    /// Converts an X coordinate relative to the input text start
    /// into a character index using the text layout's glyph positions.
    fn x_to_char_index(&self, relative_x: f32) -> usize {
        if let Some(ref tl) = self.input_layout {
            let positions = tl.char_x_positions();
            if positions.is_empty() {
                return 0;
            }
            for (i, &right_edge) in positions.iter().enumerate() {
                let left_edge = if i == 0 { 0.0 } else { positions[i - 1] };
                let mid = (left_edge + right_edge) / 2.0;
                if relative_x < mid {
                    return i;
                }
            }
            positions.len()
        } else {
            0
        }
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            let end = self.cursor_pos;
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        })
    }

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            if start != end && end <= self.input_text.len() {
                self.input_text.drain(start..end);
                self.cursor_pos = start;
            }
            self.selection_start = None;
        }
    }
}

impl Default for TagInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TagInput {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let inner_w = w - self.padding.left - self.padding.right;

        // Shape tag text layouts
        self.tag_layouts.clear();
        let mut tag_widths = Vec::new();
        for tag in &self.tags {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(13.0);
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                tag,
                &opts,
                self.tag_text_color,
                None,
            );
            let text_w = tl.size().width;
            // tag width = padding_left(8) + text + padding_right(8) + x_button(16 if removable)
            let tag_w = 8.0 + text_w + 8.0 + if self.removable { 16.0 } else { 0.0 };
            tag_widths.push(tag_w);
            self.tag_layouts.push(Some(tl));
        }

        // Shape input text layout
        let mut opts = ctx.font_options.clone();
        opts.size = Some(13.0);
        if !self.input_text.is_empty() {
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &self.input_text,
                &opts,
                colors::foreground(),
                None,
            );
            self.input_layout = Some(tl);
        } else {
            self.input_layout = None;
        }

        // Shape placeholder layout
        let tl = aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &self.placeholder,
            &opts,
            colors::muted_foreground(),
            None,
        );
        self.placeholder_layout = Some(tl);

        // Flow layout: compute tag_rects
        self.tag_rects.clear();
        let mut cx = self.padding.left;
        let mut cy = self.padding.top;

        for tag_w in &tag_widths {
            // Wrap to next line if this tag doesn't fit
            if cx + tag_w > w - self.padding.right && cx > self.padding.left {
                cx = self.padding.left;
                cy += self.tag_height + self.tag_spacing;
            }
            let tag_rect = Rect::new(cx, cy, cx + tag_w, cy + self.tag_height);
            self.tag_rects.push(tag_rect);
            cx += tag_w + self.tag_spacing;
        }

        // Input area takes remaining space on last row (min 60px)
        let input_min_w = 60.0_f32;
        let remaining = (w - self.padding.right) - cx;
        if remaining < input_min_w && cx > self.padding.left {
            // Wrap input to next line
            cy += self.tag_height + self.tag_spacing;
        }

        // Total height
        let total_h = (cy + self.tag_height + self.padding.bottom).max(self.min_height);

        let _ = inner_w; // used implicitly through w - padding calculations
        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Background
        canvas.fill_rounded_rect(rect, self.corners, self.background);

        // Border (error > focused > default)
        let border_color = if self.error {
            colors::destructive()
        } else if self.focused {
            colors::ring()
        } else {
            self.border_color
        };
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Focus ring
        if self.focused && !self.error {
            let ring_rect = Rect::new(
                rect.x1 - 1.0,
                rect.y1 - 1.0,
                rect.x2 + 1.0,
                rect.y2 + 1.0,
            );
            let ring_corners = Corners::all(self.corners.top_left + 1.0);
            canvas.stroke_rounded_rect(ring_rect, ring_corners, 1, colors::ring());
        }

        // Draw tags
        let tag_pill_corners = Corners::all(self.tag_height / 2.0);
        for (i, tag_rect) in self.tag_rects.iter().enumerate() {
            // Offset by widget position
            let tr = Rect::new(
                rect.x1 + tag_rect.x1,
                rect.y1 + tag_rect.y1,
                rect.x1 + tag_rect.x2,
                rect.y1 + tag_rect.y2,
            );

            // Tag pill background
            canvas.fill_rounded_rect(tr, tag_pill_corners, self.tag_color);

            // Tag label text
            if let Some(Some(tl)) = self.tag_layouts.get(i) {
                let ts = tl.size();
                let tx = tr.x1 + 8.0;
                let ty = tr.y1 + (self.tag_height - ts.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // X button for removable tags
            if self.removable {
                let btn_rect = self.remove_button_rect(&tr);
                let is_hovered = self.hover_remove == Some(i);
                let x_color = if is_hovered {
                    Color::new(255, 255, 255, 255)
                } else {
                    self.tag_text_color.opacity(0.7)
                };

                // Draw X as two crossing lines
                let cx = btn_rect.x1 + 8.0;
                let cy = btn_rect.y1 + 8.0;
                let half = 4.0;
                canvas.draw_line(
                    Point::new(cx - half, cy - half),
                    Point::new(cx + half, cy + half),
                    1.0,
                    x_color,
                );
                canvas.draw_line(
                    Point::new(cx + half, cy - half),
                    Point::new(cx - half, cy + half),
                    1.0,
                    x_color,
                );
            }
        }

        // Input text or placeholder
        // Determine position after last tag
        let (input_x, input_y) = if let Some(last_rect) = self.tag_rects.last() {
            let ix = rect.x1 + last_rect.x2 + self.tag_spacing;
            let remaining = rect.x2 - self.padding.right - ix;
            if remaining < 60.0 {
                // Wrap to next line
                (
                    rect.x1 + self.padding.left,
                    rect.y1 + last_rect.y2 + self.tag_spacing,
                )
            } else {
                (ix, rect.y1 + last_rect.y1)
            }
        } else {
            (rect.x1 + self.padding.left, rect.y1 + self.padding.top)
        };

        // Selection highlight
        if let Some((sel_start, sel_end)) = self.selection_range()
            && sel_start != sel_end
                && let Some(ref tl) = self.input_layout {
                    let positions = tl.char_x_positions();
                    let start_x = if sel_start > 0 && !positions.is_empty() {
                        positions[sel_start.min(positions.len()) - 1]
                    } else {
                        0.0
                    };
                    let end_x = if sel_end > 0 && !positions.is_empty() {
                        positions[sel_end.min(positions.len()) - 1]
                    } else {
                        0.0
                    };
                    canvas.fill_rect(
                        Rect::new(
                            input_x + start_x,
                            input_y,
                            input_x + end_x,
                            input_y + self.tag_height,
                        ),
                        colors::primary().opacity(0.3),
                    );
                }

        if !self.input_text.is_empty() {
            if let Some(ref tl) = self.input_layout {
                let ts = tl.size();
                let ty = input_y + (self.tag_height - ts.height) / 2.0;
                canvas.draw_text(tl, input_x as i32, ty as i32);
            }
        } else if self.tags.is_empty() {
            // Show placeholder only when no tags and no input text
            if let Some(ref tl) = self.placeholder_layout {
                let ts = tl.size();
                let ty = input_y + (self.tag_height - ts.height) / 2.0;
                canvas.draw_text(tl, input_x as i32, ty as i32);
            }
        }

        // Draw cursor when focused
        if self.focused {
            let cursor_x = if !self.input_text.is_empty() {
                if let Some(ref tl) = self.input_layout {
                    let positions = tl.char_x_positions();
                    if self.cursor_pos > 0 && !positions.is_empty() {
                        input_x + positions[self.cursor_pos.min(positions.len()) - 1]
                    } else {
                        input_x
                    }
                } else {
                    input_x
                }
            } else {
                input_x
            };
            let cursor_y = input_y;
            canvas.fill_rect(
                Rect::new(cursor_x, cursor_y, cursor_x + 1.5, cursor_y + self.tag_height),
                colors::foreground(),
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if rect.contains(&e.position) {
                    self.focused = true;

                    // Check if clicking on a tag's X button
                    if self.removable {
                        for (i, tag_rect) in self.tag_rects.iter().enumerate() {
                            let tr = Rect::new(
                                rect.x1 + tag_rect.x1,
                                rect.y1 + tag_rect.y1,
                                rect.x1 + tag_rect.x2,
                                rect.y1 + tag_rect.y2,
                            );
                            let btn = self.remove_button_rect(&tr);
                            if btn.contains(&e.position) {
                                self.remove_tag(i);
                                self.hover_remove = None;
                                self.notify_change();
                                return EventResponse {
                                    status: EventStatus::Consumed,
                                    cursor: Some(CursorIcon::Pointer),
                                    ..Default::default()
                                };
                            }
                        }
                    }

                    // Click in input area — position cursor
                    self.mouse_down = true;
                    // Compute input_x position (same logic as paint)
                    let input_x = if let Some(last_rect) = self.tag_rects.last() {
                        let ix = rect.x1 + last_rect.x2 + self.tag_spacing;
                        let remaining = rect.x2 - self.padding.right - ix;
                        if remaining < 60.0 {
                            rect.x1 + self.padding.left
                        } else {
                            ix
                        }
                    } else {
                        rect.x1 + self.padding.left
                    };
                    let relative_x = e.position.x - input_x;
                    self.cursor_pos = self.x_to_char_index(relative_x.max(0.0));
                    self.selection_start = Some(self.cursor_pos);

                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                // Click outside — unfocus
                self.focused = false;
                self.selection_start = None;
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Released =>
            {
                if self.mouse_down {
                    self.mouse_down = false;
                    if self.selection_start == Some(self.cursor_pos) {
                        self.selection_start = None;
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                // Drag selection in input
                if self.mouse_down && self.focused {
                    let input_x = if let Some(last_rect) = self.tag_rects.last() {
                        let ix = rect.x1 + last_rect.x2 + self.tag_spacing;
                        let remaining = rect.x2 - self.padding.right - ix;
                        if remaining < 60.0 {
                            rect.x1 + self.padding.left
                        } else {
                            ix
                        }
                    } else {
                        rect.x1 + self.padding.left
                    };
                    let relative_x = pos.x - input_x;
                    self.cursor_pos = self.x_to_char_index(relative_x.max(0.0));
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                if rect.contains(pos) {
                    // Track hover over X buttons
                    if self.removable {
                        let mut found = None;
                        for (i, tag_rect) in self.tag_rects.iter().enumerate() {
                            let tr = Rect::new(
                                rect.x1 + tag_rect.x1,
                                rect.y1 + tag_rect.y1,
                                rect.x1 + tag_rect.x2,
                                rect.y1 + tag_rect.y2,
                            );
                            let btn = self.remove_button_rect(&tr);
                            if btn.contains(pos) {
                                found = Some(i);
                                break;
                            }
                        }
                        self.hover_remove = found;
                    }

                    let cursor = if self.hover_remove.is_some() {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::Text
                    };

                    return EventResponse {
                        cursor: Some(cursor),
                        ..Default::default()
                    };
                }
                self.hover_remove = None;
                EventResponse::default()
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !self.focused || ch.is_control() {
                    return EventResponse::default();
                }
                // Delete selection if active
                if self.selection_range().is_some_and(|(s, e)| s != e) {
                    self.delete_selection();
                }
                self.selection_start = None;
                // Comma triggers tag creation
                if *ch == ',' {
                    let text = std::mem::take(&mut self.input_text);
                    self.cursor_pos = 0;
                    if self.try_add_tag(text) {
                        self.notify_change();
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }
                self.input_text.insert(self.cursor_pos, *ch);
                self.cursor_pos += ch.len_utf8();
                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::KeyPressed { key, modifiers }) => {
                if !self.focused {
                    return EventResponse::default();
                }
                match key {
                    Key::Enter => {
                        self.selection_start = None;
                        let text = std::mem::take(&mut self.input_text);
                        self.cursor_pos = 0;
                        if self.try_add_tag(text) {
                            self.notify_change();
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    }
                    Key::Backspace => {
                        if self.selection_range().is_some_and(|(s, e)| s != e) {
                            self.delete_selection();
                        } else if self.input_text.is_empty() {
                            // Remove last tag
                            if !self.tags.is_empty() {
                                self.tags.pop();
                                self.notify_change();
                            }
                        } else if self.cursor_pos > 0 {
                            let prev = self.input_text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            self.input_text.drain(prev..self.cursor_pos);
                            self.cursor_pos = prev;
                        }
                        self.selection_start = None;
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    }
                    Key::Left => {
                        if modifiers.shift && self.selection_start.is_none() {
                            self.selection_start = Some(self.cursor_pos);
                        } else if !modifiers.shift {
                            self.selection_start = None;
                        }
                        if self.cursor_pos > 0 {
                            self.cursor_pos = self.input_text[..self.cursor_pos]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    }
                    Key::Right => {
                        if modifiers.shift && self.selection_start.is_none() {
                            self.selection_start = Some(self.cursor_pos);
                        } else if !modifiers.shift {
                            self.selection_start = None;
                        }
                        if self.cursor_pos < self.input_text.len() {
                            self.cursor_pos = self.input_text[self.cursor_pos..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| self.cursor_pos + i)
                                .unwrap_or(self.input_text.len());
                        }
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    }
                    Key::Escape => {
                        self.focused = false;
                        self.input_text.clear();
                        self.cursor_pos = 0;
                        self.selection_start = None;
                        EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        }
                    }
                    _ => EventResponse::default(),
                }
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Tag input".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_tag() {
        let mut ti = TagInput::new();
        assert!(ti.try_add_tag("Rust".to_string()));
        assert_eq!(ti.tags, vec!["Rust"]);
    }

    #[test]
    fn remove_tag() {
        let mut ti = TagInput::new().tags(vec!["Rust", "Aurora", "UI"]);
        assert!(ti.remove_tag(1));
        assert_eq!(ti.tags, vec!["Rust", "UI"]);
    }

    #[test]
    fn duplicate_prevention() {
        let mut ti = TagInput::new();
        assert!(ti.try_add_tag("Rust".to_string()));
        assert!(!ti.try_add_tag("Rust".to_string()));
        assert!(!ti.try_add_tag("rust".to_string()));
        assert_eq!(ti.tags.len(), 1);
    }

    #[test]
    fn max_tags_enforcement() {
        let mut ti = TagInput::new().max_tags(2);
        assert!(ti.try_add_tag("one".to_string()));
        assert!(ti.try_add_tag("two".to_string()));
        assert!(!ti.try_add_tag("three".to_string()));
        assert_eq!(ti.tags.len(), 2);
    }

    #[test]
    fn backspace_removes_last() {
        let mut ti = TagInput::new().tags(vec!["Rust", "Aurora"]);
        // Simulate backspace on empty input
        assert!(ti.input_text.is_empty());
        ti.tags.pop();
        assert_eq!(ti.tags, vec!["Rust"]);
    }

    #[test]
    fn empty_input_not_added() {
        let mut ti = TagInput::new();
        assert!(!ti.try_add_tag("".to_string()));
        assert!(!ti.try_add_tag("   ".to_string()));
        assert!(ti.tags.is_empty());
    }

    #[test]
    fn default_values() {
        let ti = TagInput::new();
        assert!(ti.tags.is_empty());
        assert!(ti.input_text.is_empty());
        assert_eq!(ti.placeholder, "Add tag...");
        assert!(ti.max_tags.is_none());
        assert!(ti.removable);
        assert!(ti.width.is_none());
        assert_eq!(ti.min_height, 40.0);
        assert_eq!(ti.tag_height, 26.0);
        assert_eq!(ti.tag_spacing, 6.0);
        assert!(!ti.focused);
        assert!(!ti.error);
        assert!(ti.on_change.is_none());
        assert!(ti.hover_remove.is_none());
    }
}
