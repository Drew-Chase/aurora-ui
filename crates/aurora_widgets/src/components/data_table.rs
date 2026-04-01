use crate::layout::scrollview::ScrollState;
use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::point::Point;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

const SCROLL_SPEED: f32 = 40.0;

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
    // Virtualization (opt-in via .viewport_height())
    vp_height: Option<f32>,
    scroll_offset: f32,
    max_scroll: f32,
    visible_row_start: usize,
    visible_row_end: usize,
    buffer_rows: usize,
    scroll_state: Option<ScrollState>,
    last_mouse_pos: Point,
    scrollbar_dragging: bool,
    scrollbar_drag_anchor: f32,
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
            vp_height: None,
            scroll_offset: 0.0,
            max_scroll: 0.0,
            visible_row_start: 0,
            visible_row_end: 0,
            buffer_rows: 5,
            scroll_state: None,
            last_mouse_pos: Point::new(0.0, 0.0),
            scrollbar_dragging: false,
            scrollbar_drag_anchor: 0.0,
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

    /// Enables virtualized row rendering. Only rows visible within this
    /// height (plus a buffer zone) are laid out and painted. When not set,
    /// all rows are rendered (original behavior).
    pub fn viewport_height(mut self, height: f32) -> Self {
        self.vp_height = Some(height);
        self
    }

    /// Persistent scroll position that survives widget rebuilds.
    pub fn scroll_state(mut self, state: ScrollState) -> Self {
        self.scroll_offset = state.get();
        self.scroll_state = Some(state);
        self
    }

    /// Number of extra rows to render above/below the viewport. Default: 5.
    pub fn buffer_rows(mut self, count: usize) -> Self {
        self.buffer_rows = count;
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
            let mut tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &text,
                &opts,
                colors::foreground(),
                None,
            );
            tl.set_max_width(ctx.font_manager, col_w - 24.0);
            self.header_layouts.push(Some(tl));
        }

        // Virtualization: determine visible row range
        if let Some(ref state) = self.scroll_state {
            self.scroll_offset = state.get();
        }
        let row_count = self.sorted_indices.len();

        if let Some(vp_h) = self.vp_height {
            let total_data = self.row_height * row_count as f32;
            self.max_scroll = (total_data - vp_h).max(0.0);
            let offset = self.scroll_offset.clamp(0.0, self.max_scroll);
            let first = (offset / self.row_height).floor() as usize;
            let visible = (vp_h / self.row_height).ceil() as usize + 1;
            let last = (first + visible).min(row_count);
            self.visible_row_start = first.saturating_sub(self.buffer_rows);
            self.visible_row_end = (last + self.buffer_rows).min(row_count);
        } else {
            self.visible_row_start = 0;
            self.visible_row_end = row_count;
            self.max_scroll = 0.0;
        }

        // Cell layouts (only visible range)
        self.cell_layouts.clear();
        for display_idx in self.visible_row_start..self.visible_row_end {
            let row_idx = self.sorted_indices[display_idx];
            let row = &self.rows[row_idx];
            let mut row_layouts = Vec::new();
            for (c, cell) in row.iter().enumerate() {
                let cw = self.column_widths.get(c).copied().unwrap_or(col_w);
                let mut opts = ctx.font_options.clone();
                opts.size = Some(14.0);
                opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                let mut tl = aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    cell,
                    &opts,
                    colors::foreground(),
                    None,
                );
                tl.set_max_width(ctx.font_manager, cw - 24.0);
                row_layouts.push(Some(tl));
            }
            self.cell_layouts.push(row_layouts);
        }

        let total_h = if let Some(vp_h) = self.vp_height {
            self.header_height + vp_h
        } else {
            self.header_height + self.row_height * row_count as f32
        };
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

        // Rows — clip to data area when virtualized
        let offset = self.scroll_offset.clamp(0.0, self.max_scroll);
        let data_top = rect.y1 + self.header_height;
        let data_bottom = if self.vp_height.is_some() {
            data_top + self.vp_height.unwrap()
        } else {
            rect.y2
        };

        if self.vp_height.is_some() {
            canvas.push_clip(Rect::new(rect.x1, data_top, rect.x2, data_bottom));
        }

        for (local_idx, row_layouts) in self.cell_layouts.iter().enumerate() {
            let display_idx = self.visible_row_start + local_idx;
            let row_y = data_top + self.row_height * display_idx as f32 - offset;

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
                Rect::new(
                    rect.x1,
                    row_y + self.row_height - 1.0,
                    rect.x2,
                    row_y + self.row_height,
                ),
                self.border_color,
            );
        }

        if self.vp_height.is_some() {
            canvas.pop_clip();

            // Scrollbar
            if self.max_scroll > 0.0 {
                let sb_w = 6.0_f32;
                let track = Rect::new(rect.x2 - sb_w, data_top, rect.x2, data_bottom);
                let vp_h = self.vp_height.unwrap();
                let total = self.row_height * self.sorted_indices.len() as f32;
                let ratio = vp_h / total;
                let thumb_h = (ratio * vp_h).max(20.0);
                let scrollable = vp_h - thumb_h;
                let thumb_y = if self.max_scroll > 0.0 {
                    (offset / self.max_scroll) * scrollable
                } else {
                    0.0
                };
                let thumb = Rect::new(
                    track.x1,
                    track.y1 + thumb_y,
                    track.x2,
                    track.y1 + thumb_y + thumb_h,
                );
                canvas.fill_rounded_rect(thumb, Corners::all(sb_w / 2.0), Color::new(0, 0, 0, 100));
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(delta)) => {
                if self.vp_height.is_some() && rect.contains(&self.last_mouse_pos) {
                    let old = self.scroll_offset;
                    self.scroll_offset =
                        (self.scroll_offset - delta * SCROLL_SPEED).clamp(0.0, self.max_scroll);
                    if let Some(ref state) = self.scroll_state {
                        state.set(self.scroll_offset);
                    }
                    if (self.scroll_offset - old).abs() > 0.001 {
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    }
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                self.last_mouse_pos = *pos;

                // Scrollbar drag
                if self.scrollbar_dragging {
                    if let Some(vp_h) = self.vp_height {
                        let data_top = rect.y1 + self.header_height;
                        let total = self.row_height * self.sorted_indices.len() as f32;
                        let ratio = vp_h / total;
                        let thumb_h = (ratio * vp_h).max(20.0);
                        let scrollable = vp_h - thumb_h;
                        if scrollable > 0.0 {
                            let top = pos.y - self.scrollbar_drag_anchor - data_top;
                            let r = top / scrollable;
                            self.scroll_offset =
                                (r * self.max_scroll).clamp(0.0, self.max_scroll);
                            if let Some(ref state) = self.scroll_state {
                                state.set(self.scroll_offset);
                            }
                        }
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                if rect.contains(pos) {
                    return EventResponse {
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e)) => {
                // Release scrollbar drag
                if e.state == MouseState::Released && self.scrollbar_dragging {
                    self.scrollbar_dragging = false;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }

                if e.state != MouseState::Pressed || !rect.contains(&e.position) {
                    return EventResponse::default();
                }

                // Scrollbar thumb click
                if let Some(vp_h) = self.vp_height {
                    let data_top = rect.y1 + self.header_height;
                    let sb_w = 6.0_f32;
                    let total = self.row_height * self.sorted_indices.len() as f32;
                    let ratio = vp_h / total;
                    let thumb_h = (ratio * vp_h).max(20.0);
                    let scrollable = vp_h - thumb_h;
                    let thumb_y = if self.max_scroll > 0.0 {
                        (self.scroll_offset.clamp(0.0, self.max_scroll) / self.max_scroll)
                            * scrollable
                    } else {
                        0.0
                    };
                    let thumb = Rect::new(
                        rect.x2 - sb_w,
                        data_top + thumb_y,
                        rect.x2,
                        data_top + thumb_y + thumb_h,
                    );
                    if thumb.contains(&e.position) {
                        self.scrollbar_dragging = true;
                        self.scrollbar_drag_anchor = e.position.y - thumb.y1;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            ..Default::default()
                        };
                    }
                }

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
                                status: EventStatus::Consumed,
                                cursor: Some(CursorIcon::Pointer),
                                ..Default::default()
                            };
                        }
                        x += cw;
                    }
                    return EventResponse::default();
                }

                // Row click (account for scroll offset when virtualized)
                let offset = self.scroll_offset.clamp(0.0, self.max_scroll);
                let data_y = rect.y1 + self.header_height;
                let display_idx =
                    ((e.position.y - data_y + offset) / self.row_height) as usize;
                if display_idx < self.sorted_indices.len() {
                    let real_idx = self.sorted_indices[display_idx];
                    if let Some(ref mut cb) = self.on_row_click {
                        cb(real_idx);
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            _ => EventResponse::default(),
        }
    }
    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Table)
    }
}
