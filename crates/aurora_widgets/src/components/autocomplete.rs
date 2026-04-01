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
use std::time::Instant;

use super::colors;

const ANIM_DURATION: f32 = 0.18; // seconds

/// A text input with suggestion dropdown. The user types free text while
/// matching suggestions appear below. Selecting a suggestion fills the input.
///
/// # Example
/// ```ignore
/// AutoComplete::new()
///     .placeholder("Search...")
///     .suggestion("Apple")
///     .suggestion("Banana")
///     .suggestion("Cherry")
///     .on_select(|idx| println!("selected: {idx}"))
/// ```
pub struct AutoComplete {
    suggestions: Vec<String>,
    text: String,
    placeholder: String,
    selected: Option<usize>,
    open: bool,
    focused: bool,
    height: f32,
    item_height: f32,
    max_suggestions: usize,
    background: Color,
    border_color: Color,
    hover_bg: Color,
    corners: Corners,
    padding: Edges,
    width: Option<f32>,
    on_select: Option<Box<dyn FnMut(usize)>>,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str)>>,
    input_layout: Option<aurora_text::text_layout::TextLayout>,
    suggestion_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    filtered_indices: Vec<usize>,
    hover_index: Option<usize>,
    /// 0.0 = closed, 1.0 = open
    anim_from: f32,
    anim_to: f32,
    anim_start: Instant,
    error: bool,
}

impl AutoComplete {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            text: String::new(),
            placeholder: "Search...".to_string(),
            selected: None,
            open: false,
            focused: false,
            height: 40.0,
            item_height: 36.0,
            max_suggestions: 8,
            background: colors::background(),
            border_color: colors::input_border(),
            hover_bg: colors::accent(),
            corners: Corners::all(6.0),
            padding: Edges::new(0.0, 12.0, 0.0, 12.0),
            width: None,
            on_select: None,
            on_change: None,
            input_layout: None,
            suggestion_layouts: Vec::new(),
            filtered_indices: Vec::new(),
            hover_index: None,
            anim_from: 0.0,
            anim_to: 0.0,
            anim_start: Instant::now(),
            error: false,
        }
    }

    pub fn suggestion(mut self, label: impl Into<String>) -> Self {
        self.suggestions.push(label.into());
        self
    }

    pub fn suggestions(mut self, suggestions: Vec<impl Into<String>>) -> Self {
        self.suggestions = suggestions.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn item_height(mut self, item_height: f32) -> Self {
        self.item_height = item_height;
        self
    }

    pub fn max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
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

    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
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

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
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

    fn update_filtered(&mut self) {
        self.filtered_indices.clear();
        let search_lower = self.text.to_lowercase();
        for (i, suggestion) in self.suggestions.iter().enumerate() {
            if search_lower.is_empty() || suggestion.to_lowercase().contains(&search_lower) {
                self.filtered_indices.push(i);
                if self.filtered_indices.len() >= self.max_suggestions {
                    break;
                }
            }
        }
    }

    fn full_dropdown_height(&self) -> f32 {
        let count = self.filtered_indices.len().min(self.max_suggestions);
        8.0 + self.item_height * count as f32 + 8.0
    }

    fn dropdown_rect(&self, trigger_rect: &Rect) -> Rect {
        let h = self.full_dropdown_height();
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x2,
            trigger_rect.y2 + 4.0 + h,
        )
    }

    fn current_t(&self) -> f32 {
        let elapsed = self.anim_start.elapsed().as_secs_f32();
        let t = (elapsed / ANIM_DURATION).min(1.0);
        // CubicOut easing
        let eased = 1.0 - (1.0 - t).powi(3);
        self.anim_from + (self.anim_to - self.anim_from) * eased
    }

    fn start_anim(&mut self, opening: bool) {
        self.anim_from = self.current_t();
        self.anim_to = if opening { 1.0 } else { 0.0 };
        self.anim_start = Instant::now();
    }
}

impl Default for AutoComplete {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for AutoComplete {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        self.update_filtered();

        // Input text layout
        let display_text = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            self.text.clone()
        };

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let inner_w = w - self.padding.left - self.padding.right - 20.0;

        let text_color = if self.text.is_empty() {
            colors::muted_foreground()
        } else {
            colors::foreground()
        };

        let mut tl = aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &display_text,
            &opts,
            text_color,
            None,
        );
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.input_layout = Some(tl);

        // Suggestion layouts for filtered items
        self.suggestion_layouts.clear();
        for &idx in &self.filtered_indices {
            let suggestion = &self.suggestions[idx];
            let mut sopts = ctx.font_options.clone();
            sopts.size = Some(14.0);
            sopts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                suggestion,
                &sopts,
                colors::foreground(),
                None,
            );
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.suggestion_layouts.push(Some(tl));
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Input box (error > focused > default)
        let border_color = if self.error {
            colors::destructive()
        } else if self.focused {
            colors::ring()
        } else {
            self.border_color
        };
        canvas.fill_rounded_rect(rect, self.corners, self.background);
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Display text
        if let Some(ref tl) = self.input_layout {
            let th = tl.size().height;
            let tx = rect.x1 + self.padding.left;
            let ty = rect.y1 + (rect.height() - th) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // Chevron
        let chevron_x = rect.x2 - self.padding.right - 8.0;
        let chevron_y = rect.y1 + rect.height() / 2.0;
        canvas.draw_line(
            Point::new(chevron_x - 4.0, chevron_y - 2.0),
            Point::new(chevron_x, chevron_y + 2.0),
            1.0,
            colors::muted_foreground(),
        );
        canvas.draw_line(
            Point::new(chevron_x, chevron_y + 2.0),
            Point::new(chevron_x + 4.0, chevron_y - 2.0),
            1.0,
            colors::muted_foreground(),
        );
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        let t = self.current_t();
        if t <= 0.0 {
            return;
        }

        if self.filtered_indices.is_empty() {
            return;
        }

        // Animated dropdown panel (painted above all siblings)
        let full_h = self.full_dropdown_height();
        let visible_h = full_h * t;
        let dr = self.dropdown_rect(&rect);
        let clip = Rect::new(dr.x1, dr.y1, dr.x2, dr.y1 + visible_h);
        canvas.push_clip(clip);

        canvas.fill_rounded_rect(dr, self.corners, colors::popover());
        canvas.stroke_rounded_rect(dr, self.corners, 1, colors::border());

        let mut y = dr.y1 + 4.0;
        for (i, _) in self.filtered_indices.iter().enumerate() {
            let item_rect = Rect::new(dr.x1 + 4.0, y, dr.x2 - 4.0, y + self.item_height);
            let is_hovered = self.hover_index == Some(i);

            if is_hovered {
                let item_corners = Corners::all(4.0);
                canvas.fill_rounded_rect(item_rect, item_corners, self.hover_bg);
            }

            if let Some(Some(tl)) = self.suggestion_layouts.get(i) {
                let th = tl.size().height;
                let tx = item_rect.x1 + 8.0;
                let ty = y + (self.item_height - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            y += self.item_height;
        }

        canvas.pop_clip();
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Released =>
            {
                if rect.contains(&e.position) {
                    self.focused = true;
                    if !self.text.is_empty() || !self.suggestions.is_empty() {
                        self.open = true;
                        self.start_anim(true);
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                self.focused = false;
                self.open = false;
                self.start_anim(false);
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if rect.contains(pos) {
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Keyboard(KeyboardEvent::CharTyped(ch)) => {
                if !self.focused || ch.is_control() {
                    return EventResponse::default();
                }
                self.text.push(*ch);
                self.update_filtered();
                if !self.filtered_indices.is_empty() {
                    self.open = true;
                    self.start_anim(true);
                } else {
                    self.open = false;
                    self.start_anim(false);
                }
                self.selected = None;
                self.hover_index = None;
                if let Some(ref mut cb) = self.on_change {
                    cb(&self.text);
                }
                EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::KeyPressed { key, .. }) => {
                if !self.focused {
                    return EventResponse::default();
                }
                match key {
                    Key::Backspace => {
                        self.text.pop();
                        self.update_filtered();
                        if !self.filtered_indices.is_empty() && !self.text.is_empty() {
                            self.open = true;
                            self.start_anim(true);
                        } else if self.text.is_empty() {
                            self.open = false;
                            self.start_anim(false);
                        }
                        self.selected = None;
                        self.hover_index = None;
                        if let Some(ref mut cb) = self.on_change {
                            cb(&self.text);
                        }
                    }
                    Key::Escape => {
                        self.open = false;
                        self.start_anim(false);
                    }
                    Key::Enter => {
                        if let Some(hi) = self.hover_index
                            && hi < self.filtered_indices.len()
                        {
                            let real_idx = self.filtered_indices[hi];
                            self.selected = Some(real_idx);
                            self.text = self.suggestions[real_idx].clone();
                            self.open = false;
                            self.start_anim(false);
                            if let Some(ref mut cb) = self.on_select {
                                cb(real_idx);
                            }
                        }
                    }
                    Key::Down => {
                        let max = self.filtered_indices.len();
                        if max > 0 {
                            self.hover_index =
                                Some(self.hover_index.map(|i| (i + 1).min(max - 1)).unwrap_or(0));
                        }
                    }
                    Key::Up => {
                        if let Some(i) = self.hover_index {
                            self.hover_index = Some(i.saturating_sub(1));
                        }
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

    fn event_overlay(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        if !self.open {
            return EventResponse::default();
        }

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                if e.state == MouseState::Released {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(&e.position) {
                        let relative_y = e.position.y - dr.y1 - 4.0;
                        let idx = (relative_y / self.item_height) as usize;
                        if idx < self.filtered_indices.len() {
                            let real_idx = self.filtered_indices[idx];
                            self.selected = Some(real_idx);
                            self.text = self.suggestions[real_idx].clone();
                            self.open = false;
                            self.start_anim(false);
                            if let Some(ref mut cb) = self.on_select {
                                cb(real_idx);
                            }
                            return EventResponse {
                                status: EventStatus::Consumed,
                                ..Default::default()
                            };
                        }
                    }
                    // Click outside closes
                    self.open = false;
                    self.start_anim(false);
                    self.focused = false;
                }
                EventResponse {
                    status: EventStatus::Canceled,
                    ..Default::default()
                }
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                let dr = self.dropdown_rect(&rect);
                if dr.contains(pos) {
                    let relative_y = pos.y - dr.y1 - 4.0;
                    let idx = (relative_y / self.item_height) as usize;
                    self.hover_index = if idx < self.filtered_indices.len() {
                        Some(idx)
                    } else {
                        None
                    };
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                self.hover_index = None;
                EventResponse {
                    status: EventStatus::Canceled,
                    ..Default::default()
                }
            }
            _ => EventResponse {
                status: EventStatus::Canceled,
                ..Default::default()
            },
        }
    }

    fn needs_animation(&self) -> bool {
        self.anim_start.elapsed().as_secs_f32() < ANIM_DURATION
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        let mut info = aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::ComboBox)
            .with_expanded(self.open);
        if let Some(idx) = self.selected {
            if let Some(s) = self.suggestions.get(idx) {
                info = info.with_value(s.clone());
            }
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build an AutoComplete and return its filtered_indices after
    /// setting the text and calling update_filtered.
    fn filtered(suggestions: &[&str], text: &str) -> Vec<usize> {
        let mut ac = AutoComplete::new();
        for s in suggestions {
            ac.suggestions.push(s.to_string());
        }
        ac.text = text.to_string();
        ac.update_filtered();
        ac.filtered_indices.clone()
    }

    #[test]
    fn filter_substring_match() {
        let indices = filtered(&["Apple", "Banana", "Pineapple"], "app");
        // "app" matches "Apple" (0) and "Pineapple" (2)
        assert!(indices.contains(&0), "should match Apple");
        assert!(indices.contains(&2), "should match Pineapple");
        assert!(!indices.contains(&1), "should not match Banana");
    }

    #[test]
    fn filter_case_insensitive() {
        let indices = filtered(&["banana", "cherry"], "BANANA");
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn filter_empty_returns_all() {
        let indices = filtered(&["A", "B", "C"], "");
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn filter_no_match() {
        let indices = filtered(&["Apple", "Banana"], "xyz");
        assert!(indices.is_empty());
    }

    #[test]
    fn default_values() {
        let ac = AutoComplete::new();
        assert_eq!(ac.height, 40.0);
        assert_eq!(ac.item_height, 36.0);
        assert_eq!(ac.max_suggestions, 8);
        assert_eq!(ac.placeholder, "Search...");
        assert!(ac.suggestions.is_empty());
        assert!(ac.text.is_empty());
        assert!(!ac.open);
        assert!(!ac.focused);
        assert!(!ac.error);
        assert!(ac.width.is_none());
        assert!(ac.on_select.is_none());
        assert!(ac.on_change.is_none());
    }

    #[test]
    fn max_suggestions_capped() {
        let items: Vec<&str> = (0..20).map(|_| "item").collect();
        let mut ac = AutoComplete::new().max_suggestions(3);
        for item in &items {
            ac.suggestions.push(item.to_string());
        }
        ac.text = String::new();
        ac.update_filtered();
        assert_eq!(ac.filtered_indices.len(), 3);
    }
}
