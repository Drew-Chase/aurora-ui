use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;

use super::colors;

/// Controls how the month/year selector behaves when clicking the header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonthYearSelector {
    /// Clicking the month name shows a 3x4 month grid; clicking the year
    /// shows a year picker.
    Separate,
    /// Clicking anywhere on "Month Year" opens a single overlay with both
    /// month grid and year navigation.
    Combined,
}

/// Visual style for a cell indicator on a calendar day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndicatorStyle {
    /// A small colored dot below the day number.
    Dot,
    /// A colored ring (stroke) around the day cell.
    Ring,
}

/// A visual marker on a specific day cell with an optional tooltip.
#[derive(Clone)]
pub struct CellIndicator {
    pub day: u32,
    pub color: Color,
    pub style: IndicatorStyle,
    pub tooltip: Option<String>,
}

/// Which overlay selector is currently visible.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectorView {
    None,
    MonthGrid,
    YearGrid,
    Combined,
}

enum HeaderHit {
    None,
    Prev,
    Next,
    MonthYear,
}

/// A calendar date grid displaying a month view.
///
/// # Example
/// ```ignore
/// Calendar::new()
///     .year(2026)
///     .month(3)
///     .selected_day(25)
///     .on_select(|day| println!("selected day: {day}"))
/// ```
pub struct Calendar {
    year: u32,
    month: u32,
    selected_day: Option<u32>,
    cell_size: f32,
    header_height: f32,
    corners: Corners,
    selected_bg: Color,
    selected_fg: Color,
    today_bg: Color,
    hover_bg: Color,
    today_day: Option<u32>,
    on_select: Option<Box<dyn FnMut(u32)>>,
    on_month_change: Option<Box<dyn FnMut(u32, u32)>>,
    disabled_days: Vec<u32>,
    min_date: Option<(u32, u32, u32)>,
    max_date: Option<(u32, u32, u32)>,
    month_year_selector: Option<MonthYearSelector>,
    selector_view: SelectorView,
    selector_year_offset: i32,
    indicators: Vec<CellIndicator>,
    indicator_tooltip_day: Option<u32>,
    indicator_tooltip_layout: Option<aurora_text::text_layout::TextLayout>,
    hovered_day: Option<u32>,
    hovered_selector_month: Option<u32>,
    hovered_selector_year: Option<u32>,
    day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    weekday_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    month_label_layout: Option<aurora_text::text_layout::TextLayout>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    selector_month_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    selector_year_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    pub(crate) days_in_month: u32,
    pub(crate) first_weekday: u32,
}

impl Calendar {
    pub fn new() -> Self {
        Self {
            year: 2026,
            month: 1,
            selected_day: None,
            cell_size: 40.0,
            header_height: 40.0,
            corners: Corners::all(9999.0),
            selected_bg: colors::primary(),
            selected_fg: colors::primary_foreground(),
            today_bg: colors::accent(),
            hover_bg: colors::secondary(),
            today_day: None,
            on_select: None,
            on_month_change: None,
            disabled_days: Vec::new(),
            min_date: None,
            max_date: None,
            month_year_selector: None,
            selector_view: SelectorView::None,
            selector_year_offset: 0,
            indicators: Vec::new(),
            indicator_tooltip_day: None,
            indicator_tooltip_layout: None,
            hovered_day: None,
            hovered_selector_month: None,
            hovered_selector_year: None,
            day_layouts: Vec::new(),
            weekday_layouts: Vec::new(),
            month_label_layout: None,
            prev_layout: None,
            next_layout: None,
            selector_month_layouts: Vec::new(),
            selector_year_layouts: Vec::new(),
            days_in_month: 31,
            first_weekday: 0,
        }
    }

    pub fn year(mut self, year: u32) -> Self {
        self.year = year;
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.month = month.clamp(1, 12);
        self
    }

    pub fn selected_day(mut self, day: u32) -> Self {
        self.selected_day = Some(day);
        self
    }

    pub fn today(mut self, day: u32) -> Self {
        self.today_day = Some(day);
        self
    }

    pub fn cell_size(mut self, size: f32) -> Self {
        self.cell_size = size;
        self
    }

    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn on_select(mut self, cb: impl FnMut(u32) + 'static) -> Self {
        self.on_select = Some(Box::new(cb));
        self
    }

    pub fn on_month_change(mut self, cb: impl FnMut(u32, u32) + 'static) -> Self {
        self.on_month_change = Some(Box::new(cb));
        self
    }

    pub fn disabled_day(mut self, day: u32) -> Self {
        self.disabled_days.push(day);
        self
    }

    pub fn disabled_days(mut self, days: Vec<u32>) -> Self {
        self.disabled_days = days;
        self
    }

    pub fn min_date(mut self, year: u32, month: u32, day: u32) -> Self {
        self.min_date = Some((year, month, day));
        self
    }

    pub fn max_date(mut self, year: u32, month: u32, day: u32) -> Self {
        self.max_date = Some((year, month, day));
        self
    }

    pub fn month_year_selector(mut self, selector: MonthYearSelector) -> Self {
        self.month_year_selector = Some(selector);
        self
    }

    pub fn indicator(mut self, indicator: CellIndicator) -> Self {
        self.indicators.push(indicator);
        self
    }

    pub fn indicators(mut self, indicators: Vec<CellIndicator>) -> Self {
        self.indicators = indicators;
        self
    }

    pub(crate) fn is_day_disabled(&self, day: u32) -> bool {
        if self.disabled_days.contains(&day) {
            return true;
        }
        let date = (self.year, self.month, day);
        if let Some(min) = self.min_date
            && date < min
        {
            return true;
        }
        if let Some(max) = self.max_date
            && date > max
        {
            return true;
        }
        false
    }

    fn can_go_prev(&self) -> bool {
        if let Some((min_y, min_m, _)) = self.min_date {
            let (prev_y, prev_m) = if self.month == 1 {
                (self.year - 1, 12)
            } else {
                (self.year, self.month - 1)
            };
            (prev_y, prev_m) >= (min_y, min_m)
        } else {
            true
        }
    }

    fn can_go_next(&self) -> bool {
        if let Some((max_y, max_m, _)) = self.max_date {
            let (next_y, next_m) = if self.month == 12 {
                (self.year + 1, 1)
            } else {
                (self.year, self.month + 1)
            };
            (next_y, next_m) <= (max_y, max_m)
        } else {
            true
        }
    }

    pub(crate) fn compute_days_in_month(year: u32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    pub(crate) fn first_day_of_week(year: u32, month: u32) -> u32 {
        let t = [0u32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if month < 3 { year - 1 } else { year };
        let m = month as usize;
        (y + y / 4 - y / 100 + y / 400 + t[m - 1] + 1) % 7
    }

    pub(crate) fn month_name(month: u32) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        }
    }

    fn month_name_short(month: u32) -> &'static str {
        match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        }
    }

    fn prev_month(&mut self) {
        if !self.can_go_prev() {
            return;
        }
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
        if let Some(ref mut cb) = self.on_month_change {
            cb(self.year, self.month);
        }
    }

    fn next_month(&mut self) {
        if !self.can_go_next() {
            return;
        }
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
        if let Some(ref mut cb) = self.on_month_change {
            cb(self.year, self.month);
        }
    }

    fn total_rows(&self) -> u32 {
        let total_cells = self.first_weekday + self.days_in_month;
        total_cells.div_ceil(7)
    }

    pub(crate) fn day_at_position(
        &self,
        pos: &aurora_core::geometry::point::Point,
        rect: &Rect,
    ) -> Option<u32> {
        let grid_y = rect.y1 + self.header_height + self.cell_size;
        if pos.y < grid_y || pos.x < rect.x1 || pos.x > rect.x1 + self.cell_size * 7.0 {
            return None;
        }
        let col = ((pos.x - rect.x1) / self.cell_size) as u32;
        let row = ((pos.y - grid_y) / self.cell_size) as u32;
        if col >= 7 || row >= self.total_rows() {
            return None;
        }
        let cell_idx = row * 7 + col;
        if cell_idx < self.first_weekday {
            return None;
        }
        let day = cell_idx - self.first_weekday + 1;
        if day >= 1 && day <= self.days_in_month {
            Some(day)
        } else {
            None
        }
    }

    fn header_hit(
        &self,
        pos: &aurora_core::geometry::point::Point,
        rect: &Rect,
    ) -> HeaderHit {
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);
        if !header_rect.contains(pos) {
            return HeaderHit::None;
        }
        let third = header_rect.width() / 3.0;
        if pos.x < header_rect.x1 + third {
            HeaderHit::Prev
        } else if pos.x > header_rect.x2 - third {
            HeaderHit::Next
        } else {
            HeaderHit::MonthYear
        }
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Calendar {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.days_in_month = Self::compute_days_in_month(self.year, self.month);
        self.first_weekday = Self::first_day_of_week(self.year, self.month);

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);

        // Month label
        let month_str = format!("{} {}", Self::month_name(self.month), self.year);
        self.month_label_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            &month_str,
            &opts,
            colors::foreground(),
            None,
        ));

        // Prev/Next arrows
        let mut arrow_opts = ctx.font_options.clone();
        arrow_opts.size = Some(14.0);
        arrow_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let prev_color = if self.can_go_prev() {
            colors::foreground()
        } else {
            colors::muted_foreground()
        };
        let next_color = if self.can_go_next() {
            colors::foreground()
        } else {
            colors::muted_foreground()
        };
        self.prev_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            "<",
            &arrow_opts,
            prev_color,
            None,
        ));
        self.next_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            ">",
            &arrow_opts,
            next_color,
            None,
        ));

        // Weekday headers
        self.weekday_layouts.clear();
        let weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        for wd in &weekdays {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(12.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Medium);
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                wd,
                &opts,
                colors::muted_foreground(),
                None,
            );
            self.weekday_layouts.push(Some(tl));
        }

        // Day number layouts
        self.day_layouts.clear();
        for day in 1..=self.days_in_month {
            let is_selected = self.selected_day == Some(day);
            let is_disabled = self.is_day_disabled(day);
            let mut opts = ctx.font_options.clone();
            opts.size = Some(14.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let fg = if is_selected {
                self.selected_fg
            } else if is_disabled {
                colors::muted_foreground()
            } else {
                colors::foreground()
            };
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &day.to_string(),
                &opts,
                fg,
                None,
            );
            self.day_layouts.push(Some(tl));
        }

        // Selector layouts
        if self.selector_view != SelectorView::None {
            self.layout_selector(ctx);
        }

        // Indicator tooltip
        self.indicator_tooltip_layout = None;
        self.indicator_tooltip_day = None;
        if let Some(hovered) = self.hovered_day
            && let Some(ind) = self.indicators.iter().find(|i| i.day == hovered)
            && let Some(ref tip) = ind.tooltip
        {
                    let mut opts = ctx.font_options.clone();
                    opts.size = Some(12.0);
                    opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
                    self.indicator_tooltip_layout =
                        Some(aurora_text::text_layout::TextLayout::new(
                            ctx.font_manager,
                            tip,
                            &opts,
                            colors::popover_foreground(),
                            None,
                        ));
                    self.indicator_tooltip_day = Some(hovered);
        }

        let rows = self.total_rows();
        let total_h = self.header_height + self.cell_size + self.cell_size * rows as f32;
        Size::new(self.cell_size * 7.0, total_h)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let grid_w = self.cell_size * 7.0;

        // ── Header ──────────────────────────────────────────────────
        let header_rect =
            Rect::new(rect.x1, rect.y1, rect.x1 + grid_w, rect.y1 + self.header_height);

        if let Some(ref tl) = self.prev_layout {
            let s = tl.size();
            let tx = header_rect.x1 + 8.0;
            let ty = header_rect.y1 + (self.header_height - s.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }
        if let Some(ref tl) = self.month_label_layout {
            let s = tl.size();
            let tx = header_rect.x1 + (grid_w - s.width) / 2.0;
            let ty = header_rect.y1 + (self.header_height - s.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }
        if let Some(ref tl) = self.next_layout {
            let s = tl.size();
            let tx = header_rect.x2 - s.width - 8.0;
            let ty = header_rect.y1 + (self.header_height - s.height) / 2.0;
            canvas.draw_text(tl, tx as i32, ty as i32);
        }

        // ── Selector overlay (replaces day grid when active) ────────
        if self.selector_view != SelectorView::None {
            self.paint_selector(canvas, rect);
            return;
        }

        // ── Weekday headers ─────────────────────────────────────────
        let weekday_y = rect.y1 + self.header_height;
        for (col, layout) in self.weekday_layouts.iter().enumerate() {
            if let Some(tl) = layout {
                let s = tl.size();
                let cx = rect.x1 + col as f32 * self.cell_size + self.cell_size / 2.0;
                let tx = cx - s.width / 2.0;
                let ty = weekday_y + (self.cell_size - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }

        // ── Day grid ────────────────────────────────────────────────
        let grid_y = weekday_y + self.cell_size;
        for day in 1..=self.days_in_month {
            let cell_idx = self.first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let cx = rect.x1 + col as f32 * self.cell_size;
            let cy = grid_y + row as f32 * self.cell_size;
            let cell_rect = Rect::new(cx, cy, cx + self.cell_size, cy + self.cell_size);

            let is_selected = self.selected_day == Some(day);
            let is_today = self.today_day == Some(day);
            let is_disabled = self.is_day_disabled(day);
            let is_hovered = self.hovered_day == Some(day) && !is_disabled;

            if is_selected {
                canvas.fill_rounded_rect(cell_rect, self.corners, self.selected_bg);
            } else if is_hovered {
                canvas.fill_rounded_rect(cell_rect, self.corners, self.hover_bg);
            }

            if is_today && !is_selected {
                canvas.stroke_rounded_rect(cell_rect, self.corners, 1, self.today_bg);
            }

            if let Some(Some(tl)) = self.day_layouts.get((day - 1) as usize) {
                let s = tl.size();
                let tx = cx + (self.cell_size - s.width) / 2.0;
                let ty = cy + (self.cell_size - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }

            // Indicators
            for ind in &self.indicators {
                if ind.day != day {
                    continue;
                }
                match ind.style {
                    IndicatorStyle::Dot => {
                        let dot_y = cy + self.cell_size - 8.0;
                        let dot_cx = cx + self.cell_size / 2.0;
                        let dot_r = Rect::new(
                            dot_cx - 2.0,
                            dot_y - 2.0,
                            dot_cx + 2.0,
                            dot_y + 2.0,
                        );
                        canvas.fill_rounded_rect(dot_r, Corners::all(9999.0), ind.color);
                    }
                    IndicatorStyle::Ring => {
                        let inset = 2.0;
                        let ring_r = Rect::new(
                            cx + inset,
                            cy + inset,
                            cx + self.cell_size - inset,
                            cy + self.cell_size - inset,
                        );
                        canvas.stroke_rounded_rect(ring_r, self.corners, 1, ind.color);
                    }
                }
            }
        }
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        // Indicator tooltip
        if let (Some(day), Some(tl)) =
            (self.indicator_tooltip_day, &self.indicator_tooltip_layout)
        {
            let cell_idx = self.first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let grid_y = rect.y1 + self.header_height + self.cell_size;
            let cx = rect.x1 + col as f32 * self.cell_size;
            let cy = grid_y + row as f32 * self.cell_size;

            let s = tl.size();
            let pad = 6.0;
            let tip_w = s.width + pad * 2.0;
            let tip_h = s.height + pad * 2.0;
            let tip_x = cx + (self.cell_size - tip_w) / 2.0;
            let tip_y = cy - tip_h - 4.0;
            let tip_rect = Rect::new(tip_x, tip_y, tip_x + tip_w, tip_y + tip_h);

            canvas.fill_rounded_rect(tip_rect, Corners::all(4.0), colors::popover());
            canvas.stroke_rounded_rect(tip_rect, Corners::all(4.0), 1, colors::border());
            canvas.draw_text(tl, (tip_x + pad) as i32, (tip_y + pad) as i32);
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
                // ── Selector click ──────────────────────────────────
                if self.selector_view != SelectorView::None {
                    return self.handle_selector_click(e, &rect);
                }

                // ── Header ──────────────────────────────────────────
                match self.header_hit(&e.position, &rect) {
                    HeaderHit::Prev if self.can_go_prev() => {
                        self.prev_month();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    HeaderHit::Next if self.can_go_next() => {
                        self.next_month();
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    HeaderHit::MonthYear if self.month_year_selector.is_some() => {
                        let selector = self.month_year_selector.unwrap();
                        self.selector_view = match selector {
                            MonthYearSelector::Separate => {
                                let header_rect = Rect::new(
                                    rect.x1,
                                    rect.y1,
                                    rect.x2,
                                    rect.y1 + self.header_height,
                                );
                                let mid = header_rect.x1 + header_rect.width() / 2.0;
                                if e.position.x < mid {
                                    SelectorView::MonthGrid
                                } else {
                                    SelectorView::YearGrid
                                }
                            }
                            MonthYearSelector::Combined => SelectorView::Combined,
                        };
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    HeaderHit::Prev | HeaderHit::Next | HeaderHit::MonthYear | HeaderHit::None => {}
                }

                // ── Day grid ────────────────────────────────────────
                if let Some(day) = self.day_at_position(&e.position, &rect)
                    && !self.is_day_disabled(day)
                {
                    self.selected_day = Some(day);
                    if let Some(ref mut cb) = self.on_select {
                        cb(day);
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseMoveEvent(pos)) => {
                if !rect.contains(pos) {
                    self.hovered_day = None;
                    self.hovered_selector_month = None;
                    self.hovered_selector_year = None;
                    return EventResponse::default();
                }

                // ── Selector hover ──────────────────────────────────
                if self.selector_view != SelectorView::None {
                    return self.handle_selector_hover(pos, &rect);
                }

                // ── Header ──────────────────────────────────────────
                match self.header_hit(pos, &rect) {
                    HeaderHit::Prev if self.can_go_prev() => {
                        self.hovered_day = None;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    HeaderHit::Next if self.can_go_next() => {
                        self.hovered_day = None;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    HeaderHit::MonthYear if self.month_year_selector.is_some() => {
                        self.hovered_day = None;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Pointer),
                            ..Default::default()
                        };
                    }
                    HeaderHit::Prev | HeaderHit::Next | HeaderHit::MonthYear => {
                        self.hovered_day = None;
                        return EventResponse {
                            status: EventStatus::Consumed,
                            cursor: Some(CursorIcon::Default),
                            ..Default::default()
                        };
                    }
                    HeaderHit::None => {}
                }

                // ── Weekday row — not clickable ─────────────────────
                let weekday_y = rect.y1 + self.header_height;
                let grid_y = weekday_y + self.cell_size;
                if pos.y >= weekday_y && pos.y < grid_y {
                    self.hovered_day = None;
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: Some(CursorIcon::Default),
                        ..Default::default()
                    };
                }

                // ── Day grid hover ──────────────────────────────────
                if let Some(day) = self.day_at_position(pos, &rect) {
                    let disabled = self.is_day_disabled(day);
                    self.hovered_day = if disabled { None } else { Some(day) };
                    return EventResponse {
                        status: EventStatus::Consumed,
                        cursor: if disabled {
                            Some(CursorIcon::Default)
                        } else {
                            Some(CursorIcon::Pointer)
                        },
                        ..Default::default()
                    };
                }

                self.hovered_day = None;
                EventResponse {
                    status: EventStatus::Consumed,
                    cursor: Some(CursorIcon::Default),
                    ..Default::default()
                }
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group)
            .with_label("Calendar".to_string())
    }
}

// ─── Selector paint / event helpers ─────────────────────────────────────────

impl Calendar {
    fn layout_selector(&mut self, ctx: &mut LayoutCtx) {
        self.selector_month_layouts.clear();
        for m in 1..=12u32 {
            let mut opts = ctx.font_options.clone();
            opts.size = Some(13.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let fg = if m == self.month {
                self.selected_fg
            } else {
                colors::foreground()
            };
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                Self::month_name_short(m),
                &opts,
                fg,
                None,
            );
            self.selector_month_layouts.push(Some(tl));
        }

        self.selector_year_layouts.clear();
        let base_year = self.year as i32 + self.selector_year_offset;
        for i in 0..20i32 {
            let y = base_year - 10 + i;
            let mut opts = ctx.font_options.clone();
            opts.size = Some(13.0);
            opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            let fg = if y as u32 == self.year {
                self.selected_fg
            } else {
                colors::foreground()
            };
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                &y.to_string(),
                &opts,
                fg,
                None,
            );
            self.selector_year_layouts.push(Some(tl));
        }
    }

    fn paint_selector(&self, canvas: &mut Canvas, rect: Rect) {
        let grid_y = rect.y1 + self.header_height;
        let grid_w = self.cell_size * 7.0;
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;

        let show_months = matches!(
            self.selector_view,
            SelectorView::MonthGrid | SelectorView::Combined
        );
        let show_years = matches!(
            self.selector_view,
            SelectorView::YearGrid | SelectorView::Combined
        );

        if show_months && show_years {
            let half_h = grid_h / 2.0;
            self.paint_month_grid(canvas, rect.x1, grid_y, grid_w, half_h);
            self.paint_year_grid(canvas, rect.x1, grid_y + half_h, grid_w, half_h);
        } else if show_months {
            self.paint_month_grid(canvas, rect.x1, grid_y, grid_w, grid_h);
        } else if show_years {
            self.paint_year_grid(canvas, rect.x1, grid_y, grid_w, grid_h);
        }
    }

    fn paint_month_grid(&self, canvas: &mut Canvas, x: f32, y: f32, w: f32, h: f32) {
        let cell_w = w / 3.0;
        let cell_h = h / 4.0;
        for i in 0..12 {
            let row = i / 3;
            let col = i % 3;
            let cx = x + col as f32 * cell_w;
            let cy = y + row as f32 * cell_h;
            let cell_rect = Rect::new(cx, cy, cx + cell_w, cy + cell_h);
            let m = i as u32 + 1;

            if m == self.month {
                canvas.fill_rounded_rect(cell_rect, Corners::all(6.0), self.selected_bg);
            } else if self.hovered_selector_month == Some(m) {
                canvas.fill_rounded_rect(cell_rect, Corners::all(6.0), self.hover_bg);
            }

            if let Some(Some(tl)) = self.selector_month_layouts.get(i) {
                let s = tl.size();
                let tx = cx + (cell_w - s.width) / 2.0;
                let ty = cy + (cell_h - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }
    }

    fn paint_year_grid(&self, canvas: &mut Canvas, x: f32, y: f32, w: f32, h: f32) {
        let cell_w = w / 4.0;
        let cell_h = h / 5.0;
        let base_year = self.year as i32 + self.selector_year_offset;

        for i in 0..20 {
            let row = i / 4;
            let col = i % 4;
            let cx = x + col as f32 * cell_w;
            let cy = y + row as f32 * cell_h;
            let cell_rect = Rect::new(cx, cy, cx + cell_w, cy + cell_h);
            let yr = (base_year - 10 + i as i32) as u32;

            if yr == self.year {
                canvas.fill_rounded_rect(cell_rect, Corners::all(6.0), self.selected_bg);
            } else if self.hovered_selector_year == Some(yr) {
                canvas.fill_rounded_rect(cell_rect, Corners::all(6.0), self.hover_bg);
            }

            if let Some(Some(tl)) = self.selector_year_layouts.get(i) {
                let s = tl.size();
                let tx = cx + (cell_w - s.width) / 2.0;
                let ty = cy + (cell_h - s.height) / 2.0;
                canvas.draw_text(tl, tx as i32, ty as i32);
            }
        }
    }

    fn handle_selector_click(
        &mut self,
        e: &aurora_core::kmi::mouse::MouseClickEvent,
        rect: &Rect,
    ) -> EventResponse {
        let grid_y = rect.y1 + self.header_height;
        let grid_w = self.cell_size * 7.0;
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;

        // Click in header → close selector
        if e.position.y < grid_y {
            self.selector_view = SelectorView::None;
            return EventResponse {
                status: EventStatus::Consumed,
                ..Default::default()
            };
        }

        let show_months = matches!(
            self.selector_view,
            SelectorView::MonthGrid | SelectorView::Combined
        );
        let show_years = matches!(
            self.selector_view,
            SelectorView::YearGrid | SelectorView::Combined
        );

        if show_months && show_years {
            let half_h = grid_h / 2.0;
            if e.position.y < grid_y + half_h {
                if let Some(m) = self.month_at_pos(&e.position, rect.x1, grid_y, grid_w, half_h) {
                    self.month = m;
                    self.selector_view = SelectorView::None;
                    if let Some(ref mut cb) = self.on_month_change {
                        cb(self.year, self.month);
                    }
                    return EventResponse {
                        status: EventStatus::Consumed,
                        ..Default::default()
                    };
                }
            } else if let Some(y) =
                self.year_at_pos(&e.position, rect.x1, grid_y + half_h, grid_w, half_h)
            {
                self.year = y;
                self.selector_view = SelectorView::None;
                if let Some(ref mut cb) = self.on_month_change {
                    cb(self.year, self.month);
                }
                return EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                };
            }
        } else if show_months {
            if let Some(m) = self.month_at_pos(&e.position, rect.x1, grid_y, grid_w, grid_h) {
                self.month = m;
                self.selector_view = SelectorView::None;
                if let Some(ref mut cb) = self.on_month_change {
                    cb(self.year, self.month);
                }
                return EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                };
            }
        } else if show_years {
            if let Some(y) = self.year_at_pos(&e.position, rect.x1, grid_y, grid_w, grid_h) {
                self.year = y;
                self.selector_view = SelectorView::None;
                if let Some(ref mut cb) = self.on_month_change {
                    cb(self.year, self.month);
                }
                return EventResponse {
                    status: EventStatus::Consumed,
                    ..Default::default()
                };
            }
        }

        EventResponse::default()
    }

    fn handle_selector_hover(
        &mut self,
        pos: &aurora_core::geometry::point::Point,
        rect: &Rect,
    ) -> EventResponse {
        let grid_y = rect.y1 + self.header_height;
        let grid_w = self.cell_size * 7.0;
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;

        self.hovered_selector_month = None;
        self.hovered_selector_year = None;

        let show_months = matches!(
            self.selector_view,
            SelectorView::MonthGrid | SelectorView::Combined
        );
        let show_years = matches!(
            self.selector_view,
            SelectorView::YearGrid | SelectorView::Combined
        );

        if show_months && show_years {
            let half_h = grid_h / 2.0;
            if pos.y >= grid_y && pos.y < grid_y + half_h {
                self.hovered_selector_month =
                    self.month_at_pos(pos, rect.x1, grid_y, grid_w, half_h);
            } else if pos.y >= grid_y + half_h {
                self.hovered_selector_year =
                    self.year_at_pos(pos, rect.x1, grid_y + half_h, grid_w, half_h);
            }
        } else if show_months {
            self.hovered_selector_month = self.month_at_pos(pos, rect.x1, grid_y, grid_w, grid_h);
        } else if show_years {
            self.hovered_selector_year = self.year_at_pos(pos, rect.x1, grid_y, grid_w, grid_h);
        }

        let hovering =
            self.hovered_selector_month.is_some() || self.hovered_selector_year.is_some();
        EventResponse {
            status: EventStatus::Consumed,
            cursor: Some(if hovering {
                CursorIcon::Pointer
            } else {
                CursorIcon::Default
            }),
            ..Default::default()
        }
    }

    fn month_at_pos(
        &self,
        pos: &aurora_core::geometry::point::Point,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Option<u32> {
        if pos.x < x || pos.x > x + w || pos.y < y || pos.y > y + h {
            return None;
        }
        let col = ((pos.x - x) / (w / 3.0)) as u32;
        let row = ((pos.y - y) / (h / 4.0)) as u32;
        if col < 3 && row < 4 {
            let m = row * 3 + col + 1;
            if m >= 1 && m <= 12 {
                return Some(m);
            }
        }
        None
    }

    fn year_at_pos(
        &self,
        pos: &aurora_core::geometry::point::Point,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Option<u32> {
        if pos.x < x || pos.x > x + w || pos.y < y || pos.y > y + h {
            return None;
        }
        let col = ((pos.x - x) / (w / 4.0)) as u32;
        let row = ((pos.y - y) / (h / 5.0)) as u32;
        if col < 4 && row < 5 {
            let idx = row * 4 + col;
            let base_year = self.year as i32 + self.selector_year_offset;
            let yr = base_year - 10 + idx as i32;
            if yr > 0 {
                return Some(yr as u32);
            }
        }
        None
    }

    // Accessors for CalendarRange (different names to avoid conflict with builders)
    pub(crate) fn header_height_val(&self) -> f32 {
        self.header_height
    }

    pub(crate) fn cell_size_val(&self) -> f32 {
        self.cell_size
    }

    pub(crate) fn day_layouts(&self) -> &[Option<aurora_text::text_layout::TextLayout>] {
        &self.day_layouts
    }

    pub(crate) fn selected_fg_val(&self) -> Color {
        self.selected_fg
    }
}
