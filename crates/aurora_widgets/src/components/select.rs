use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A select dropdown input that shows a list of options.
///
/// # Example
/// ```ignore
/// Select::new()
///     .placeholder("Select an option")
///     .option("Option A")
///     .option("Option B")
///     .option("Option C")
///     .on_change(|idx| println!("selected: {idx}"))
/// ```
pub struct Select {
    options: Vec<String>,
    selected: Option<usize>,
    placeholder: String,
    open: bool,
    height: f32,
    item_height: f32,
    dropdown_width: Option<f32>,
    background: Color,
    border_color: Color,
    hover_bg: Color,
    corners: Corners,
    padding: Edges,
    disabled: bool,
    width: Option<f32>,
    on_change: Option<Box<dyn FnMut(usize)>>,
    selected_layout: Option<aurora_text::text_layout::TextLayout>,
    option_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    hover_index: Option<usize>,
}

impl Select {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            placeholder: "Select...".to_string(),
            open: false,
            height: 40.0,
            item_height: 36.0,
            dropdown_width: None,
            background: colors::BACKGROUND,
            border_color: colors::INPUT_BORDER,
            hover_bg: colors::ACCENT,
            corners: Corners::all(6.0),
            padding: Edges::new(0.0, 12.0, 0.0, 12.0),
            disabled: false,
            width: None,
            on_change: None,
            selected_layout: None,
            option_layouts: Vec::new(),
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

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_change = Some(Box::new(cb));
        self
    }

    fn dropdown_rect(&self, trigger_rect: &Rect) -> Rect {
        let w = self.dropdown_width.unwrap_or(trigger_rect.width());
        let h = 8.0 + self.item_height * self.options.len() as f32 + 8.0;
        Rect::new(
            trigger_rect.x1,
            trigger_rect.y2 + 4.0,
            trigger_rect.x1 + w,
            trigger_rect.y2 + 4.0 + h,
        )
    }
}

impl Default for Select {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Select {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);

        // Display text
        let display_text = if let Some(idx) = self.selected {
            self.options.get(idx).cloned().unwrap_or_default()
        } else {
            self.placeholder.clone()
        };

        let _text_color = if self.selected.is_some() {
            colors::FOREGROUND
        } else {
            colors::MUTED_FOREGROUND
        };

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let inner_w = w - self.padding.left - self.padding.right - 20.0;
        let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &display_text, &opts, colors::FOREGROUND, None);
        tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
        self.selected_layout = Some(tl);

        // Option layouts
        self.option_layouts.clear();
        for option in &self.options {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, option, &opts, colors::FOREGROUND, None);
            tl.set_max_width(ctx.font_manager, inner_w.max(0.0));
            self.option_layouts.push(Some(tl));
        }

        Size::new(w, self.height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        // Trigger box
        let border_color = if self.open { colors::RING } else { self.border_color };
        canvas.fill_rounded_rect(rect, self.corners, self.background);
        canvas.stroke_rounded_rect(rect, self.corners, 1, border_color);

        // Selected/placeholder text
        if let Some(ref tl) = self.selected_layout {
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
            1,
            colors::MUTED_FOREGROUND,
        );
        canvas.draw_line(
            Point::new(chevron_x, chevron_y + 2.0),
            Point::new(chevron_x + 4.0, chevron_y - 2.0),
            1,
            colors::MUTED_FOREGROUND,
        );

        if !self.open {
            return;
        }

        // Dropdown panel
        let dr = self.dropdown_rect(&rect);
        canvas.fill_rounded_rect(dr, self.corners, colors::POPOVER);
        canvas.stroke_rounded_rect(dr, self.corners, 1, colors::BORDER);

        let mut y = dr.y1 + 4.0;
        for (i, _option) in self.options.iter().enumerate() {
            let item_rect = Rect::new(dr.x1 + 4.0, y, dr.x2 - 4.0, y + self.item_height);
            let is_selected = self.selected == Some(i);
            let is_hovered = self.hover_index == Some(i);

            if is_hovered || is_selected {
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
        if self.disabled {
            return EventResponse::default();
        }

        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed =>
            {
                if rect.contains(&e.position) {
                    self.open = !self.open;
                    self.hover_index = None;
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                if self.open {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(&e.position) {
                        let relative_y = e.position.y - dr.y1 - 4.0;
                        let idx = (relative_y / self.item_height) as usize;
                        if idx < self.options.len() {
                            self.selected = Some(idx);
                            self.open = false;
                            if let Some(ref mut cb) = self.on_change {
                                cb(idx);
                            }
                            return EventResponse {
                                handled: true,
                                ..Default::default()
                            };
                        }
                    }
                    self.open = false;
                    return EventResponse {
                        handled: true,
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if self.open {
                    let dr = self.dropdown_rect(&rect);
                    if dr.contains(pos) {
                        let relative_y = pos.y - dr.y1 - 4.0;
                        let idx = (relative_y / self.item_height) as usize;
                        self.hover_index = if idx < self.options.len() { Some(idx) } else { None };
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
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
}
