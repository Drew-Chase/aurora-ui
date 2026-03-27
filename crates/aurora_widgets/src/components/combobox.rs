use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::keyboard::{Key, KeyboardEvent};
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A searchable select (combobox). The user types to filter options, then
/// selects from matching results.
///
/// # Example
/// ```ignore
/// Combobox::new()
///     .placeholder("Search...")
///     .option("Apple")
///     .option("Banana")
///     .option("Cherry")
///     .on_select(|idx| println!("selected: {idx}"))
/// ```
pub struct Combobox {
    options: Vec<String>,
    search: String,
    selected: Option<usize>,
    placeholder: String,
    open: bool,
    focused: bool,
    height: f32,
    item_height: f32,
    max_dropdown_items: usize,
    background: Color,
    border_color: Color,
    hover_bg: Color,
    corners: Corners,
    padding: Edges,
    width: Option<f32>,
    on_select: Option<Box<dyn FnMut(usize)>>,
    input_layout: Option<aurora_text::text_layout::TextLayout>,
    option_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    filtered_indices: Vec<usize>,
    hover_index: Option<usize>,
}

impl Combobox {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            search: String::new(),
            selected: None,
            placeholder: "Search...".to_string(),
            open: false,
            focused: false,
            height: 40.0,
            item_height: 36.0,
            max_dropdown_items: 8,
            background: colors::background(),
            border_color: colors::input_border(),
            hover_bg: colors::accent(),
            corners: Corners::all(6.0),
            padding: Edges::new(0.0, 12.0, 0.0, 12.0),
            width: None,
            on_select: None,
            input_layout: None,
            option_layouts: Vec::new(),
            filtered_indices: Vec::new(),
            hover_index: None,
        }
    }

    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(label.into());
        self
    }

    pub fn options(mut self, options: Vec<impl Into<String>>) -> Self {
        self.options = options.into_iter().map(|o| o.into()).collect();
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
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

    fn update_filtered(&mut self) {
        self.filtered_indices.clear();
        let search_lower = self.search.to_lowercase();
        for (i, option) in self.options.iter().enumerate() {
            if search_lower.is_empty() || option.to_lowercase().contains(&search_lower) {
                self.filtered_indices.push(i);
                if self.filtered_indices.len() >= self.max_dropdown_items {
                    break;
                }
            }
        }
    }

    fn dropdown_rect(&self, trigger_rect: &Rect) -> Rect {
        let count = self.filtered_indices.len().min(self.max_dropdown_items);
        let h = 8.0 + self.item_height * count as f32 + 8.0;
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x2,
            trigger_rect.y2 + 4.0 + h,
        )
    }
}

impl Default for Combobox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Combobox {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        self.update_filtered();

        // Input text layout
        let display_text = if self.search.is_empty() {
            if let Some(idx) = self.selected {
                self.options.get(idx).cloned().unwrap_or_default()
            } else {
                self.placeholder.clone()
            }
        } else {
            self.search.clone()
        };

        let _text_color = if !self.search.is_empty() || self.selected.is_some() {
            colors::foreground()
        } else {
            colors::muted_foreground()
        };

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let inner_w = w - self.padding.left - self.padding.right - 20.0;
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &display_text, &opts, colors::foreground(), None);
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.input_layout = Some(tl);

        // Option layouts for filtered items
        self.option_layouts.clear();
        for &idx in &self.filtered_indices {
            let option = &self.options[idx];
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, option, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.option_layouts.push(Some(tl));
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Input box
        let border_color = if self.focused { colors::ring() } else { self.border_color };
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

        if !self.open || self.filtered_indices.is_empty() {
            return;
        }

        // Dropdown
        let dr = self.dropdown_rect(&rect);
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

            if let Some(Some(tl)) = self.option_layouts.get(i) {
                let th = tl.size().height;
                let tx = item_rect.x1 + 8.0;
                let ty = y + (self.item_height - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            y += self.item_height;
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
                    self.open = true;
                    self.search.clear();
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Text),
                        ..Default::default()
                    };
                }
                if self.open {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(&e.position) {
                        let relative_y = e.position.y - dr.y1 - 4.0;
                        let idx = (relative_y / self.item_height) as usize;
                        if idx < self.filtered_indices.len() {
                            let real_idx = self.filtered_indices[idx];
                            self.selected = Some(real_idx);
                            self.open = false;
                            self.search.clear();
                            if let Some(ref mut cb) = self.on_select {
                                cb(real_idx);
                            }
                            return EventResponse {
                                handled: true,
                                ..Default::default()
                            };
                        }
                    }
                    self.open = false;
                    self.focused = false;
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                self.focused = false;
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.open {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(pos) {
                        let relative_y = pos.y - dr.y1 - 4.0;
                        let idx = (relative_y / self.item_height) as usize;
                        self.hover_index = if idx < self.filtered_indices.len() { Some(idx) } else { None };
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    } else {
                        self.hover_index = None;
                    }
                }
                if rect.contains(pos) {
                    return EventResponse {
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
                self.search.push(*ch);
                self.open = true;
                self.selected = None;
                EventResponse {
                    handled: true,
                    ..Default::default()
                }
            }
            WidgetEvent::Keyboard(KeyboardEvent::KeyPressed { key, .. }) => {
                if !self.focused {
                    return EventResponse::default();
                }
                match key {
                    Key::Backspace => {
                        self.search.pop();
                        self.open = true;
                    }
                    Key::Escape => {
                        self.open = false;
                        self.search.clear();
                    }
                    Key::Enter => {
                        if let Some(hi) = self.hover_index
                            && hi < self.filtered_indices.len()
                        {
                            let real_idx = self.filtered_indices[hi];
                            self.selected = Some(real_idx);
                            self.open = false;
                            self.search.clear();
                            if let Some(ref mut cb) = self.on_select {
                                cb(real_idx);
                            }
                        }
                    }
                    Key::Down => {
                        let max = self.filtered_indices.len();
                        if max > 0 {
                            self.hover_index = Some(
                                self.hover_index.map(|i| (i + 1).min(max - 1)).unwrap_or(0),
                            );
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
                    handled: true,
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
}
