use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_core::kmi::WidgetEvent;
use aurora_render::canvas::Canvas;

use super::colors;

/// Sort direction for a column.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SortDirection {
    #[default]
    None,
    Ascending,
    Descending,
}

/// A sortable, filterable data table with headers and rows.
///
/// Click a column header to sort by that column.
///
/// # Example
/// ```ignore
/// DataTable::new()
///     .column("Name")
///     .column("Email")
///     .column("Status")
///     .row(vec!["Alice", "alice@example.com", "Active"])
///     .row(vec!["Bob", "bob@example.com", "Inactive"])
///     .on_sort(|col, dir| println!("sort col {col} {:?}", dir))
/// ```
pub struct DataTable {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    sorted_indices: Vec<usize>,
    sort_column: Option<usize>,
    sort_direction: SortDirection,
    row_height: f32,
    header_height: f32,
    border_color: Color,
    header_bg: Color,
    _header_fg: Color,
    striped: bool,
    width: Option<f32>,
    column_widths: Vec<f32>,
    on_sort: Option<Box<dyn FnMut(usize, SortDirection)>>,
    on_row_click: Option<Box<dyn FnMut(usize)>>,
    header_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    cell_layouts: Vec<Vec<Option<aurora_text::text_layout::TextLayout>>>,
}

impl DataTable {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            sorted_indices: Vec::new(),
            sort_column: None,
            sort_direction: SortDirection::None,
            row_height: 48.0,
            header_height: 48.0,
            border_color: colors::border(),
            header_bg: colors::muted(),
            _header_fg: colors::muted_foreground(),
            striped: true,
            width: None,
            column_widths: Vec::new(),
            on_sort: None,
            on_row_click: None,
            header_layouts: Vec::new(),
            cell_layouts: Vec::new(),
        }
    }

    pub fn column(mut self, name: impl Into<String>) -> Self {
        self.columns.push(name.into());
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

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_sort(mut self, cb: impl FnMut(usize, SortDirection) + 'static) -> Self {
        self.on_sort = Some(Box::new(cb));
        self
    }

    pub fn on_row_click(mut self, cb: impl FnMut(usize) + 'static) -> Self {
        self.on_row_click = Some(Box::new(cb));
        self
    }

    fn apply_sort(&mut self) {
        self.sorted_indices = (0..self.rows.len()).collect();
        if let Some(col) = self.sort_column {
            match self.sort_direction {
                SortDirection::Ascending => {
                    self.sorted_indices.sort_by(|&a, &b| {
                        let va = self.rows[a].get(col).map(|s| s.as_str()).unwrap_or("");
                        let vb = self.rows[b].get(col).map(|s| s.as_str()).unwrap_or("");
                        va.cmp(vb)
                    });
                }
                SortDirection::Descending => {
                    self.sorted_indices.sort_by(|&a, &b| {
                        let va = self.rows[a].get(col).map(|s| s.as_str()).unwrap_or("");
                        let vb = self.rows[b].get(col).map(|s| s.as_str()).unwrap_or("");
                        vb.cmp(va)
                    });
                }
                SortDirection::None => {}
            }
        }
    }
}

impl Default for DataTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DataTable {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let col_count = self.columns.len();
        if col_count == 0 {
            return Size::new(w, 0.0);
        }

        let col_w = w / col_count as f32;
        self.column_widths = vec![col_w; col_count];

        self.apply_sort();

        // Header layouts
        self.header_layouts.clear();
        for (i, col) in self.columns.iter().enumerate() {
            let suffix = match (self.sort_column, self.sort_direction) {
                (Some(sc), SortDirection::Ascending) if sc == i => " ^",
                (Some(sc), SortDirection::Descending) if sc == i => " v",
                _ => "",
            };
            let text = format!("{}{}", col, suffix);
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, &text, &opts, colors::foreground(), None);
            tl.set_max_width(ctx.font_manager, col_w - 24.0);
            self.header_layouts.push(Some(tl));
        }

        // Cell layouts (in sorted order)
        self.cell_layouts.clear();
        for &row_idx in &self.sorted_indices {
            let row = &self.rows[row_idx];
            let mut row_layouts = Vec::new();
            for (c, cell) in row.iter().enumerate() {
                let cw = self.column_widths.get(c).copied().unwrap_or(col_w);
                let mut opts = ctx.font_options.clone();
                opts.size = Some(14.0);
                opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                let mut tl = aurora_text::text_layout::TextLayout::new(ctx.font_manager, cell, &opts, colors::foreground(), None);
                tl.set_max_width(ctx.font_manager, cw - 24.0);
                row_layouts.push(Some(tl));
            }
            self.cell_layouts.push(row_layouts);
        }

        let total_h = self.header_height + self.row_height * self.sorted_indices.len() as f32;
        Size::new(w, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        if self.columns.is_empty() {
            return;
        }

        // Header
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);
        canvas.fill_rect(header_rect, self.header_bg);

        for (col, _) in self.column_widths.iter().enumerate() {
            let cell_x = rect.x1 + self.column_widths[..col].iter().sum::<f32>();
            if let Some(Some(tl)) = self.header_layouts.get(col) {
                let th = tl.size().height;
                let tx = cell_x + 12.0;
                let ty = rect.y1 + (self.header_height - th) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }

        canvas.fill_rect(
            Rect::new(rect.x1, header_rect.y2 - 1.0, rect.x2, header_rect.y2),
            self.border_color,
        );

        // Rows
        for (display_idx, row_layouts) in self.cell_layouts.iter().enumerate() {
            let row_y = rect.y1 + self.header_height + self.row_height * display_idx as f32;

            let bg = if self.striped && display_idx % 2 == 1 {
                colors::muted()
            } else {
                colors::background()
            };
            canvas.fill_rect(
                Rect::new(rect.x1, row_y, rect.x2, row_y + self.row_height),
                bg,
            );

            for (col, _) in self.column_widths.iter().enumerate() {
                let cell_x = rect.x1 + self.column_widths[..col].iter().sum::<f32>();
                if let Some(Some(tl)) = row_layouts.get(col) {
                    let th = tl.size().height;
                    let tx = cell_x + 12.0;
                    let ty = row_y + (self.row_height - th) / 2.0;
                    canvas.draw_text(tl, tx as i32, ty as i32);
                }
            }

            canvas.fill_rect(
                Rect::new(rect.x1, row_y + self.row_height - 1.0, rect.x2, row_y + self.row_height),
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
                // Header click => sort
                if e.position.y < rect.y1 + self.header_height {
                    let mut x = rect.x1;
                    for (col, cw) in self.column_widths.iter().enumerate() {
                        if e.position.x >= x && e.position.x < x + cw {
                            if self.sort_column == Some(col) {
                                self.sort_direction = match self.sort_direction {
                                    SortDirection::None => SortDirection::Ascending,
                                    SortDirection::Ascending => SortDirection::Descending,
                                    SortDirection::Descending => SortDirection::None,
                                };
                            } else {
                                self.sort_column = Some(col);
                                self.sort_direction = SortDirection::Ascending;
                            }
                            if let Some(ref mut cb) = self.on_sort {
                                cb(col, self.sort_direction);
                            }
                            return EventResponse {
                                handled: true,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        x += cw;
                    }
                    return EventResponse::default();
                }

                // Row click
                let data_y = rect.y1 + self.header_height;
                let display_idx = ((e.position.y - data_y) / self.row_height) as usize;
                if display_idx < self.sorted_indices.len() {
                    let real_idx = self.sorted_indices[display_idx];
                    if let Some(ref mut cb) = self.on_row_click {
                        cb(real_idx);
                    }
                    return EventResponse {
                        handled: true,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) if rect.contains(pos) => {
                EventResponse {
                    cursor: Some(CursorIcon::Pointer),
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }
#[cfg(feature = "a11y")]    fn access_info(&self) -> aurora_a11y::NodeInfo {        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Table)    }
}
