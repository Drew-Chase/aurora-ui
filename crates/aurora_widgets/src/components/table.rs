use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// A data table with headers and rows.
///
/// # Example
/// ```ignore
/// Table::new()
///     .headers(vec!["Name", "Email", "Role"])
///     .row(vec!["Alice", "alice@example.com", "Admin"])
///     .row(vec!["Bob", "bob@example.com", "User"])
/// ```
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    row_height: f32,
    header_height: f32,
    column_widths: Vec<f32>,
    border_color: Color,
    header_bg: Color,
    _header_fg: Color,
    row_bg: Color,
    alt_row_bg: Color,
    striped: bool,
    width: Option<f32>,
    on_row_click: Option<Box<dyn FnMut(usize)>>,
    header_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    cell_layouts: Vec<Vec<Option<aurora_text::text_layout::TextLayout>>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            row_height: 48.0,
            header_height: 48.0,
            column_widths: Vec::new(),
            border_color: colors::border(),
            header_bg: colors::muted(),
            _header_fg: colors::muted_foreground(),
            row_bg: colors::background(),
            alt_row_bg: colors::muted(),
            striped: true,
            width: None,
            on_row_click: None,
            header_layouts: Vec::new(),
            cell_layouts: Vec::new(),
        }
    }

    pub fn headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.headers = headers.into_iter().map(|h| h.into()).collect();
        self
    }

    pub fn row(mut self, row: Vec<impl Into<String>>) -> Self {
        self.rows.push(row.into_iter().map(|c| c.into()).collect());
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_row_click(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_row_click = Some(Box::new(cb));
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Table {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let col_count = self.headers.len().max(
            self.rows.iter().map(|r| r.len()).max().unwrap_or(0),
        );

        if col_count == 0 {
            return Size::new(w, 0.0);
        }

        let col_w = w / col_count as f32;
        self.column_widths = vec![col_w; col_count];

        // Layout header text
        self.header_layouts.clear();
        for header in &self.headers {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, header, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, col_w - 24.0);
            self.header_layouts.push(Some(tl));
        }

        // Layout cell text
        self.cell_layouts.clear();
        for row in &self.rows {
            let mut row_layouts = Vec::new();
            for cell in row {
                let mut opts = ctx.font_options.clone();
                opts.size = Some(14.0);
                opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, cell, &opts, colors::foreground(), None);
                tl.set_max_width(ctx.font_manager, col_w - 24.0);
                row_layouts.push(Some(tl));
            }
            self.cell_layouts.push(row_layouts);
        }

        let total_h = self.header_height + self.row_height * self.rows.len() as f32;
        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let col_count = self.column_widths.len();
        if col_count == 0 {
            return;
        }

        // Header row
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);
        canvas.fill_rect(header_rect, self.header_bg);

        for (col, col_w) in self.column_widths.iter().enumerate() {
            let cell_x = rect.x1 + self.column_widths[..col].iter().sum::<f32>();
            if let Some(Some(tl)) = self.header_layouts.get(col) {
                let th = tl.size().height;
                let tx = cell_x + 12.0;
                let ty = rect.y1 + (self.header_height - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
            // Column separator
            if col < col_count - 1 {
                let sep_x = cell_x + col_w;
                canvas.fill_rect(
                    Rect::new(sep_x - 0.5, rect.y1, sep_x + 0.5, rect.y1 + self.header_height),
                    self.border_color,
                );
            }
        }

        // Header bottom border
        canvas.fill_rect(
            Rect::new(rect.x1, header_rect.y2 - 1.0, rect.x2, header_rect.y2),
            self.border_color,
        );

        // Data rows
        for (row_idx, row_layouts) in self.cell_layouts.iter().enumerate() {
            let row_y = rect.y1 + self.header_height + self.row_height * row_idx as f32;
            let row_rect = Rect::new(rect.x1, row_y, rect.x2, row_y + self.row_height);

            // Alternating row background
            let bg = if self.striped && row_idx % 2 == 1 {
                self.alt_row_bg
            } else {
                self.row_bg
            };
            canvas.fill_rect(row_rect, bg);

            for (col, _col_w) in self.column_widths.iter().enumerate() {
                let cell_x = rect.x1 + self.column_widths[..col].iter().sum::<f32>();
                if let Some(Some(tl)) = row_layouts.get(col) {
                    let th = tl.size().height;
                    let tx = cell_x + 12.0;
                    let ty = row_y + (self.row_height - th) / 2.0;
                    canvas.draw_text(tl, tx as i32, ty as i32);
                }
            }

            // Row bottom border
            canvas.fill_rect(
                Rect::new(rect.x1, row_rect.y2 - 1.0, rect.x2, row_rect.y2),
                self.border_color,
            );
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                let data_y = rect.y1 + self.header_height;
                if e.position.y >= data_y {
                    let row_idx = ((e.position.y - data_y) / self.row_height) as usize;
                    if row_idx < self.rows.len() {
                        if let Some(ref mut cb) = self.on_row_click {
                            cb(row_idx);
                        }
                        return EventResponse {
                            handled: true,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                let data_y = rect.y1 + self.header_height;
                if pos.y >= data_y {
                    let row_idx = ((pos.y - data_y) / self.row_height) as usize;
                    if row_idx < self.rows.len() {
                        return EventResponse {
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Table)    }
}
