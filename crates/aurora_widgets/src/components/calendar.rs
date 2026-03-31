use crate::widgets::{EventResponse, EventStatus, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::WidgetEvent;
use aurora_core::kmi::cursor_icon::CursorIcon;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use std::time::Instant;

use super::colors;

// ─── Animation helpers ──────────────────────────────────────────────────────

const ANIM_DURATION: f32 = 0.20;
const SLIDE_DURATION: f32 = 0.25;
const SELECTOR_ITEM_H: f32 = 32.0;
const SELECTOR_YEAR_COUNT: i32 = 21;

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn anim_t(from: f32, to: f32, start: Instant, duration: f32) -> f32 {
    let elapsed = start.elapsed().as_secs_f32();
    let raw = (elapsed / duration).min(1.0);
    let eased = ease_out_cubic(raw);
    from + (to - from) * eased
}

fn anim_active(start: Instant, duration: f32) -> bool {
    start.elapsed().as_secs_f32() < duration
}

// ─── Public types ───────────────────────────────────────────────────────────

/// Controls how the month/year selector behaves when clicking the header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonthYearSelector {
    /// Clicking the month name opens the month column; clicking the year
    /// opens the year column.
    Separate,
    /// Clicking anywhere on "Month Year" opens both columns side by side.
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

// ─── Internal enums ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectorView {
    None,
    MonthColumn,
    YearColumn,
    Combined,
}

enum HeaderHit {
    None,
    Prev,
    Next,
    MonthYear,
}

// ─── Calendar widget ────────────────────────────────────────────────────────

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
    indicators: Vec<CellIndicator>,
    indicator_tooltip_day: Option<u32>,
    indicator_tooltip_layout: Option<aurora_text::text_layout::TextLayout>,
    hovered_day: Option<u32>,
    hovered_selector_month: Option<u32>,
    hovered_selector_year: Option<u32>,
    // Animation: selector open/close
    selector_anim_from: f32,
    selector_anim_to: f32,
    selector_anim_start: Instant,
    // Animation: month slide
    month_slide_from: f32,
    month_slide_start: Instant,
    // Selector scroll offsets
    month_scroll_offset: f32,
    year_scroll_offset: f32,
    // Precomputed layouts
    day_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    prev_trailing_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    next_trailing_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    weekday_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    month_label_layout: Option<aurora_text::text_layout::TextLayout>,
    prev_layout: Option<aurora_text::text_layout::TextLayout>,
    next_layout: Option<aurora_text::text_layout::TextLayout>,
    selector_month_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    selector_year_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    pub(crate) days_in_month: u32,
    pub(crate) first_weekday: u32,
    prev_month_days: u32,
}

impl Calendar {
    pub fn new() -> Self {
        let now = Instant::now();
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
            indicators: Vec::new(),
            indicator_tooltip_day: None,
            indicator_tooltip_layout: None,
            hovered_day: None,
            hovered_selector_month: None,
            hovered_selector_year: None,
            selector_anim_from: 0.0,
            selector_anim_to: 0.0,
            selector_anim_start: now,
            month_slide_from: 0.0,
            month_slide_start: now,
            month_scroll_offset: 0.0,
            year_scroll_offset: 0.0,
            day_layouts: Vec::new(),
            prev_trailing_layouts: Vec::new(),
            next_trailing_layouts: Vec::new(),
            weekday_layouts: Vec::new(),
            month_label_layout: None,
            prev_layout: None,
            next_layout: None,
            selector_month_layouts: Vec::new(),
            selector_year_layouts: Vec::new(),
            days_in_month: 31,
            first_weekday: 0,
            prev_month_days: 31,
        }
    }

    // ── Builder methods ─────────────────────────────────────────────────

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

    // ── Internal helpers ────────────────────────────────────────────────

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
            let (py, pm) = if self.month == 1 { (self.year - 1, 12) } else { (self.year, self.month - 1) };
            (py, pm) >= (min_y, min_m)
        } else {
            true
        }
    }

    fn can_go_next(&self) -> bool {
        if let Some((max_y, max_m, _)) = self.max_date {
            let (ny, nm) = if self.month == 12 { (self.year + 1, 1) } else { (self.year, self.month + 1) };
            (ny, nm) <= (max_y, max_m)
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
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "",
        }
    }

    fn month_name_short(month: u32) -> &'static str {
        match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "",
        }
    }

    fn prev_month(&mut self) {
        if !self.can_go_prev() { return; }
        let grid_w = self.cell_size * 7.0;
        self.month_slide_from = grid_w;
        self.month_slide_start = Instant::now();
        if self.month == 1 { self.month = 12; self.year -= 1; } else { self.month -= 1; }
        if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
    }

    fn next_month(&mut self) {
        if !self.can_go_next() { return; }
        let grid_w = self.cell_size * 7.0;
        self.month_slide_from = -grid_w;
        self.month_slide_start = Instant::now();
        if self.month == 12 { self.month = 1; self.year += 1; } else { self.month += 1; }
        if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
    }

    fn total_rows(&self) -> u32 {
        (self.first_weekday + self.days_in_month).div_ceil(7)
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
        if col >= 7 || row >= self.total_rows() { return None; }
        let cell_idx = row * 7 + col;
        if cell_idx < self.first_weekday { return None; }
        let day = cell_idx - self.first_weekday + 1;
        if (1..=self.days_in_month).contains(&day) { Some(day) } else { None }
    }

    fn header_hit(&self, pos: &aurora_core::geometry::point::Point, rect: &Rect) -> HeaderHit {
        let header_rect = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);
        if !header_rect.contains(pos) { return HeaderHit::None; }
        let third = header_rect.width() / 3.0;
        if pos.x < header_rect.x1 + third {
            HeaderHit::Prev
        } else if pos.x > header_rect.x2 - third {
            HeaderHit::Next
        } else {
            HeaderHit::MonthYear
        }
    }

    fn open_selector(&mut self, view: SelectorView) {
        self.selector_anim_from = self.selector_anim_t();
        self.selector_anim_to = 1.0;
        self.selector_anim_start = Instant::now();
        self.selector_view = view;

        // Center scroll on current month/year
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;
        let viewport_h = grid_h;
        self.month_scroll_offset = ((self.month as f32 - 1.0) * SELECTOR_ITEM_H - viewport_h / 2.0 + SELECTOR_ITEM_H / 2.0).max(0.0);
        let year_center = SELECTOR_YEAR_COUNT as f32 / 2.0;
        self.year_scroll_offset = (year_center * SELECTOR_ITEM_H - viewport_h / 2.0 + SELECTOR_ITEM_H / 2.0).max(0.0);
    }

    fn close_selector(&mut self) {
        self.selector_anim_from = self.selector_anim_t();
        self.selector_anim_to = 0.0;
        self.selector_anim_start = Instant::now();
    }

    fn selector_anim_t(&self) -> f32 {
        anim_t(self.selector_anim_from, self.selector_anim_to, self.selector_anim_start, ANIM_DURATION)
    }

    fn month_slide_x(&self) -> f32 {
        anim_t(self.month_slide_from, 0.0, self.month_slide_start, SLIDE_DURATION)
    }

    fn selector_grid_rect(&self, rect: &Rect) -> Rect {
        let grid_y = rect.y1 + self.header_height;
        let grid_w = self.cell_size * 7.0;
        let grid_h = self.cell_size * self.total_rows() as f32 + self.cell_size;
        Rect::new(rect.x1, grid_y, rect.x1 + grid_w, grid_y + grid_h)
    }

    // ── Selector hit helpers ────────────────────────────────────────────

    fn selector_month_at(&self, pos: &aurora_core::geometry::point::Point, col_rect: &Rect) -> Option<u32> {
        if !col_rect.contains(pos) { return None; }
        let y_in_col = pos.y - col_rect.y1 + self.month_scroll_offset;
        let idx = (y_in_col / SELECTOR_ITEM_H) as u32;
        if (1..=12).contains(&(idx + 1)) { Some(idx + 1) } else { None }
    }

    fn selector_year_at(&self, pos: &aurora_core::geometry::point::Point, col_rect: &Rect) -> Option<u32> {
        if !col_rect.contains(pos) { return None; }
        let y_in_col = pos.y - col_rect.y1 + self.year_scroll_offset;
        let idx = (y_in_col / SELECTOR_ITEM_H) as i32;
        let base_year = self.year as i32 - SELECTOR_YEAR_COUNT / 2;
        let yr = base_year + idx;
        if yr > 0 { Some(yr as u32) } else { None }
    }
}

impl Default for Calendar {
    fn default() -> Self { Self::new() }
}

// ─── Widget implementation ──────────────────────────────────────────────────

impl Widget for Calendar {
    fn layout(&mut self, _available: Size, ctx: &mut LayoutCtx) -> Size {
        self.days_in_month = Self::compute_days_in_month(self.year, self.month);
        self.first_weekday = Self::first_day_of_week(self.year, self.month);
        let (py, pm) = if self.month == 1 { (self.year - 1, 12) } else { (self.year, self.month - 1) };
        self.prev_month_days = Self::compute_days_in_month(py, pm);

        let mut opts = ctx.font_options.clone();
        opts.size = Some(14.0);
        opts.weight = Some(aurora_text::font_options::FontWeight::Medium);

        // Month label
        let month_str = format!("{} {}", Self::month_name(self.month), self.year);
        self.month_label_layout = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager, &month_str, &opts, colors::foreground(), None,
        ));

        // Prev/Next arrows
        let mut arrow_opts = ctx.font_options.clone();
        arrow_opts.size = Some(14.0);
        arrow_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        let prev_c = if self.can_go_prev() { colors::foreground() } else { colors::muted_foreground() };
        let next_c = if self.can_go_next() { colors::foreground() } else { colors::muted_foreground() };
        self.prev_layout = Some(aurora_text::text_layout::TextLayout::new(ctx.font_manager, "<", &arrow_opts, prev_c, None));
        self.next_layout = Some(aurora_text::text_layout::TextLayout::new(ctx.font_manager, ">", &arrow_opts, next_c, None));

        // Weekday headers
        self.weekday_layouts.clear();
        let weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        for wd in &weekdays {
            let mut o = ctx.font_options.clone();
            o.size = Some(12.0);
            o.weight = Some(aurora_text::font_options::FontWeight::Medium);
            self.weekday_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, wd, &o, colors::muted_foreground(), None,
            )));
        }

        // Day number layouts
        self.day_layouts.clear();
        let mut day_opts = ctx.font_options.clone();
        day_opts.size = Some(14.0);
        day_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        for day in 1..=self.days_in_month {
            let is_selected = self.selected_day == Some(day);
            let is_disabled = self.is_day_disabled(day);
            let fg = if is_selected { self.selected_fg } else if is_disabled { colors::muted_foreground() } else { colors::foreground() };
            self.day_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &day.to_string(), &day_opts, fg, None,
            )));
        }

        // Previous month trailing days
        self.prev_trailing_layouts.clear();
        let mut trail_opts = ctx.font_options.clone();
        trail_opts.size = Some(14.0);
        trail_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
        for i in 0..self.first_weekday {
            let d = self.prev_month_days - self.first_weekday + i + 1;
            self.prev_trailing_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &d.to_string(), &trail_opts, colors::muted_foreground(), None,
            )));
        }

        // Next month trailing days
        self.next_trailing_layouts.clear();
        let last_cell = self.first_weekday + self.days_in_month;
        let total_cells = self.total_rows() * 7;
        for i in 0..(total_cells - last_cell) {
            let d = i + 1;
            self.next_trailing_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, &d.to_string(), &trail_opts, colors::muted_foreground(), None,
            )));
        }

        // Selector column layouts
        if self.selector_view != SelectorView::None {
            self.selector_month_layouts.clear();
            let mut s_opts = ctx.font_options.clone();
            s_opts.size = Some(13.0);
            s_opts.weight = Some(aurora_text::font_options::FontWeight::Normal);
            for m in 1..=12u32 {
                let fg = if m == self.month { self.selected_fg } else { colors::foreground() };
                self.selector_month_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager, Self::month_name_short(m), &s_opts, fg, None,
                )));
            }
            self.selector_year_layouts.clear();
            let base = self.year as i32 - SELECTOR_YEAR_COUNT / 2;
            for i in 0..SELECTOR_YEAR_COUNT {
                let yr = base + i;
                let fg = if yr as u32 == self.year { self.selected_fg } else { colors::foreground() };
                self.selector_year_layouts.push(Some(aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager, &yr.to_string(), &s_opts, fg, None,
                )));
            }
        }

        // Indicator tooltip
        self.indicator_tooltip_layout = None;
        self.indicator_tooltip_day = None;
        if let Some(hovered) = self.hovered_day
            && let Some(ind) = self.indicators.iter().find(|i| i.day == hovered)
            && let Some(ref tip) = ind.tooltip
        {
            let mut o = ctx.font_options.clone();
            o.size = Some(12.0);
            o.weight = Some(aurora_text::font_options::FontWeight::Normal);
            self.indicator_tooltip_layout = Some(aurora_text::text_layout::TextLayout::new(
                ctx.font_manager, tip, &o, colors::popover_foreground(), None,
            ));
            self.indicator_tooltip_day = Some(hovered);
        }

        let rows = self.total_rows();
        Size::new(self.cell_size * 7.0, self.header_height + self.cell_size + self.cell_size * rows as f32)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let grid_w = self.cell_size * 7.0;

        // ── Header ──────────────────────────────────────────────────
        if let Some(ref tl) = self.prev_layout {
            let s = tl.size();
            canvas.draw_text(tl, (rect.x1 + 8.0) as i32, (rect.y1 + (self.header_height - s.height) / 2.0) as i32);
        }
        if let Some(ref tl) = self.month_label_layout {
            let s = tl.size();
            canvas.draw_text(tl, (rect.x1 + (grid_w - s.width) / 2.0) as i32, (rect.y1 + (self.header_height - s.height) / 2.0) as i32);
        }
        if let Some(ref tl) = self.next_layout {
            let s = tl.size();
            canvas.draw_text(tl, (rect.x1 + grid_w - s.width - 8.0) as i32, (rect.y1 + (self.header_height - s.height) / 2.0) as i32);
        }

        // ── Selector overlay (replaces grid when active) ────────────
        let sel_t = self.selector_anim_t();
        if self.selector_view != SelectorView::None || sel_t > 0.01 {
            self.paint_selector_columns(canvas, rect, sel_t);
            if sel_t > 0.99 { return; }
        }

        // ── Month slide offset ──────────────────────────────────────
        let slide_x = self.month_slide_x();
        let weekday_y = rect.y1 + self.header_height;
        let grid_y = weekday_y + self.cell_size;

        // Clip to grid area for slide animation
        let grid_clip = Rect::new(rect.x1, weekday_y, rect.x1 + grid_w, rect.y2);
        canvas.push_clip(grid_clip);

        // ── Weekday headers ─────────────────────────────────────────
        for (col, layout) in self.weekday_layouts.iter().enumerate() {
            if let Some(tl) = layout {
                let s = tl.size();
                let cx = rect.x1 + col as f32 * self.cell_size + self.cell_size / 2.0 + slide_x;
                canvas.draw_text(tl, (cx - s.width / 2.0) as i32, (weekday_y + (self.cell_size - s.height) / 2.0) as i32);
            }
        }

        // ── Previous month trailing days ────────────────────────────
        for (i, layout) in self.prev_trailing_layouts.iter().enumerate() {
            if let Some(tl) = layout {
                let col = i as u32;
                let cx = rect.x1 + col as f32 * self.cell_size + slide_x;
                let cy = grid_y;
                let s = tl.size();
                canvas.draw_text(tl, (cx + (self.cell_size - s.width) / 2.0) as i32, (cy + (self.cell_size - s.height) / 2.0) as i32);
            }
        }

        // ── Day grid ────────────────────────────────────────────────
        for day in 1..=self.days_in_month {
            let cell_idx = self.first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let cx = rect.x1 + col as f32 * self.cell_size + slide_x;
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
                canvas.draw_text(tl, (cx + (self.cell_size - s.width) / 2.0) as i32, (cy + (self.cell_size - s.height) / 2.0) as i32);
            }

            // Indicators
            for ind in &self.indicators {
                if ind.day != day { continue; }
                match ind.style {
                    IndicatorStyle::Dot => {
                        let dy = cy + self.cell_size - 8.0;
                        let dcx = cx + self.cell_size / 2.0;
                        canvas.fill_rounded_rect(Rect::new(dcx - 2.0, dy - 2.0, dcx + 2.0, dy + 2.0), Corners::all(9999.0), ind.color);
                    }
                    IndicatorStyle::Ring => {
                        let ins = 2.0;
                        canvas.stroke_rounded_rect(Rect::new(cx + ins, cy + ins, cx + self.cell_size - ins, cy + self.cell_size - ins), self.corners, 1, ind.color);
                    }
                }
            }
        }

        // ── Next month trailing days ────────────────────────────────
        let last_cell = self.first_weekday + self.days_in_month;
        for (i, layout) in self.next_trailing_layouts.iter().enumerate() {
            if let Some(tl) = layout {
                let cell_idx = last_cell + i as u32;
                let row = cell_idx / 7;
                let col = cell_idx % 7;
                let cx = rect.x1 + col as f32 * self.cell_size + slide_x;
                let cy = grid_y + row as f32 * self.cell_size;
                let s = tl.size();
                canvas.draw_text(tl, (cx + (self.cell_size - s.width) / 2.0) as i32, (cy + (self.cell_size - s.height) / 2.0) as i32);
            }
        }

        canvas.pop_clip();
    }

    fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
        if let (Some(day), Some(tl)) = (self.indicator_tooltip_day, &self.indicator_tooltip_layout) {
            let cell_idx = self.first_weekday + day - 1;
            let row = cell_idx / 7;
            let col = cell_idx % 7;
            let grid_y = rect.y1 + self.header_height + self.cell_size;
            let cx = rect.x1 + col as f32 * self.cell_size;
            let cy = grid_y + row as f32 * self.cell_size;
            let s = tl.size();
            let pad = 6.0;
            let tw = s.width + pad * 2.0;
            let th = s.height + pad * 2.0;
            let tx = cx + (self.cell_size - tw) / 2.0;
            let ty = cy - th - 4.0;
            let tr = Rect::new(tx, ty, tx + tw, ty + th);
            canvas.fill_rounded_rect(tr, Corners::all(4.0), colors::popover());
            canvas.stroke_rounded_rect(tr, Corners::all(4.0), 1, colors::border());
            canvas.draw_text(tl, (tx + pad) as i32, (ty + pad) as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] { &[] }

    fn needs_animation(&self) -> bool {
        anim_active(self.selector_anim_start, ANIM_DURATION)
            || anim_active(self.month_slide_start, SLIDE_DURATION)
    }

    fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
        match event {
            WidgetEvent::Mouse(MouseEvent::MouseScrollEvent(delta)) => {
                // Scroll selector columns
                if self.selector_view != SelectorView::None {
                    let grid_rect = self.selector_grid_rect(&rect);
                    // Determine which column the cursor is over using last hovered state
                    let show_months = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
                    let show_years = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);
                    let mid_x = grid_rect.x1 + grid_rect.width() / 2.0;

                    if show_months && show_years {
                        // Heuristic: scroll whichever column has a hovered item
                        if self.hovered_selector_month.is_some() {
                            self.month_scroll_offset = (self.month_scroll_offset - delta * SELECTOR_ITEM_H).max(0.0);
                        } else if self.hovered_selector_year.is_some() {
                            self.year_scroll_offset = (self.year_scroll_offset - delta * SELECTOR_ITEM_H).max(0.0);
                        }
                    } else if show_months {
                        self.month_scroll_offset = (self.month_scroll_offset - delta * SELECTOR_ITEM_H).max(0.0);
                    } else if show_years {
                        self.year_scroll_offset = (self.year_scroll_offset - delta * SELECTOR_ITEM_H).max(0.0);
                    }
                    return EventResponse { status: EventStatus::Consumed, ..Default::default() };
                }
                EventResponse::default()
            }
            WidgetEvent::Mouse(MouseEvent::MouseClickEvent(e))
                if e.state == MouseState::Pressed && rect.contains(&e.position) =>
            {
                // ── Selector click ──────────────────────────────────
                if self.selector_view != SelectorView::None && self.selector_anim_t() > 0.5 {
                    return self.handle_selector_click(e, &rect);
                }

                // ── Header ──────────────────────────────────────────
                match self.header_hit(&e.position, &rect) {
                    HeaderHit::Prev if self.can_go_prev() => {
                        self.prev_month();
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    HeaderHit::Next if self.can_go_next() => {
                        self.next_month();
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    HeaderHit::MonthYear if self.month_year_selector.is_some() => {
                        if self.selector_view != SelectorView::None {
                            self.close_selector();
                        } else {
                            let sel = self.month_year_selector.unwrap();
                            let view = match sel {
                                MonthYearSelector::Separate => {
                                    let hr = Rect::new(rect.x1, rect.y1, rect.x2, rect.y1 + self.header_height);
                                    if e.position.x < hr.x1 + hr.width() / 2.0 { SelectorView::MonthColumn } else { SelectorView::YearColumn }
                                }
                                MonthYearSelector::Combined => SelectorView::Combined,
                            };
                            self.open_selector(view);
                        }
                        return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
                    }
                    HeaderHit::Prev | HeaderHit::Next | HeaderHit::MonthYear | HeaderHit::None => {}
                }

                // ── Day grid ────────────────────────────────────────
                if let Some(day) = self.day_at_position(&e.position, &rect)
                    && !self.is_day_disabled(day)
                {
                    self.selected_day = Some(day);
                    if let Some(ref mut cb) = self.on_select { cb(day); }
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() };
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
                if self.selector_view != SelectorView::None && self.selector_anim_t() > 0.5 {
                    return self.handle_selector_hover(pos, &rect);
                }

                // ── Header ──────────────────────────────────────────
                match self.header_hit(pos, &rect) {
                    HeaderHit::Prev if self.can_go_prev() => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::Next if self.can_go_next() => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::MonthYear if self.month_year_selector.is_some() => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Pointer), ..Default::default() }; }
                    HeaderHit::Prev | HeaderHit::Next | HeaderHit::MonthYear => { self.hovered_day = None; return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Default), ..Default::default() }; }
                    HeaderHit::None => {}
                }

                // ── Weekday row ─────────────────────────────────────
                let wy = rect.y1 + self.header_height;
                let gy = wy + self.cell_size;
                if pos.y >= wy && pos.y < gy {
                    self.hovered_day = None;
                    return EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Default), ..Default::default() };
                }

                // ── Day grid hover ──────────────────────────────────
                if let Some(day) = self.day_at_position(pos, &rect) {
                    let disabled = self.is_day_disabled(day);
                    self.hovered_day = if disabled { None } else { Some(day) };
                    return EventResponse { status: EventStatus::Consumed, cursor: if disabled { Some(CursorIcon::Default) } else { Some(CursorIcon::Pointer) }, ..Default::default() };
                }

                self.hovered_day = None;
                EventResponse { status: EventStatus::Consumed, cursor: Some(CursorIcon::Default), ..Default::default() }
            }
            _ => EventResponse::default(),
        }
    }

    #[cfg(feature = "a11y")]
    fn access_info(&self) -> aurora_a11y::NodeInfo {
        aurora_a11y::NodeInfo::new(aurora_a11y::accesskit::Role::Group).with_label("Calendar".to_string())
    }
}

// ─── Selector column paint / event ──────────────────────────────────────────

impl Calendar {
    fn paint_selector_columns(&self, canvas: &mut Canvas, rect: Rect, t: f32) {
        let grid_rect = self.selector_grid_rect(&rect);
        let full_h = grid_rect.height();
        let anim_h = full_h * t;

        // Clip to animated height (slide-down effect)
        let clip = Rect::new(grid_rect.x1, grid_rect.y1, grid_rect.x2, grid_rect.y1 + anim_h);
        canvas.push_clip(clip);

        let show_months = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
        let show_years = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);

        if show_months && show_years {
            let half_w = grid_rect.width() / 2.0;
            let month_col = Rect::new(grid_rect.x1, grid_rect.y1, grid_rect.x1 + half_w, grid_rect.y2);
            let year_col = Rect::new(grid_rect.x1 + half_w, grid_rect.y1, grid_rect.x2, grid_rect.y2);
            self.paint_month_column(canvas, &month_col);
            // Separator
            canvas.fill_rect(Rect::new(month_col.x2 - 0.5, grid_rect.y1 + 4.0, month_col.x2 + 0.5, grid_rect.y1 + anim_h - 4.0), colors::border());
            self.paint_year_column(canvas, &year_col);
        } else if show_months {
            self.paint_month_column(canvas, &grid_rect);
        } else if show_years {
            self.paint_year_column(canvas, &grid_rect);
        }

        canvas.pop_clip();
    }

    fn paint_month_column(&self, canvas: &mut Canvas, col_rect: &Rect) {
        canvas.push_clip(*col_rect);
        for (i, layout) in self.selector_month_layouts.iter().enumerate() {
            let m = i as u32 + 1;
            let cy = col_rect.y1 + i as f32 * SELECTOR_ITEM_H - self.month_scroll_offset;
            let item_rect = Rect::new(col_rect.x1, cy, col_rect.x2, cy + SELECTOR_ITEM_H);

            if m == self.month {
                canvas.fill_rounded_rect(item_rect, Corners::all(4.0), self.selected_bg);
            } else if self.hovered_selector_month == Some(m) {
                canvas.fill_rounded_rect(item_rect, Corners::all(4.0), self.hover_bg);
            }

            if let Some(Some(tl)) = self.selector_month_layouts.get(i) {
                let s = tl.size();
                canvas.draw_text(tl, (col_rect.x1 + (col_rect.width() - s.width) / 2.0) as i32, (cy + (SELECTOR_ITEM_H - s.height) / 2.0) as i32);
            }
        }
        canvas.pop_clip();
    }

    fn paint_year_column(&self, canvas: &mut Canvas, col_rect: &Rect) {
        canvas.push_clip(*col_rect);
        let base = self.year as i32 - SELECTOR_YEAR_COUNT / 2;
        for (i, layout) in self.selector_year_layouts.iter().enumerate() {
            let yr = (base + i as i32) as u32;
            let cy = col_rect.y1 + i as f32 * SELECTOR_ITEM_H - self.year_scroll_offset;
            let item_rect = Rect::new(col_rect.x1, cy, col_rect.x2, cy + SELECTOR_ITEM_H);

            if yr == self.year {
                canvas.fill_rounded_rect(item_rect, Corners::all(4.0), self.selected_bg);
            } else if self.hovered_selector_year == Some(yr) {
                canvas.fill_rounded_rect(item_rect, Corners::all(4.0), self.hover_bg);
            }

            if let Some(Some(tl)) = self.selector_year_layouts.get(i) {
                let s = tl.size();
                canvas.draw_text(tl, (col_rect.x1 + (col_rect.width() - s.width) / 2.0) as i32, (cy + (SELECTOR_ITEM_H - s.height) / 2.0) as i32);
            }
        }
        canvas.pop_clip();
    }

    fn handle_selector_click(&mut self, e: &aurora_core::kmi::mouse::MouseClickEvent, rect: &Rect) -> EventResponse {
        let grid_rect = self.selector_grid_rect(rect);

        // Click in header → close
        if e.position.y < grid_rect.y1 {
            self.close_selector();
            return EventResponse { status: EventStatus::Consumed, ..Default::default() };
        }

        let show_months = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
        let show_years = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);

        if show_months && show_years {
            let half_w = grid_rect.width() / 2.0;
            let month_col = Rect::new(grid_rect.x1, grid_rect.y1, grid_rect.x1 + half_w, grid_rect.y2);
            let year_col = Rect::new(grid_rect.x1 + half_w, grid_rect.y1, grid_rect.x2, grid_rect.y2);
            if let Some(m) = self.selector_month_at(&e.position, &month_col) {
                self.month = m;
                self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
            if let Some(y) = self.selector_year_at(&e.position, &year_col) {
                self.year = y;
                self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
        } else if show_months {
            if let Some(m) = self.selector_month_at(&e.position, &grid_rect) {
                self.month = m;
                self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
        } else if show_years {
            if let Some(y) = self.selector_year_at(&e.position, &grid_rect) {
                self.year = y;
                self.close_selector();
                if let Some(ref mut cb) = self.on_month_change { cb(self.year, self.month); }
                return EventResponse { status: EventStatus::Consumed, ..Default::default() };
            }
        }

        EventResponse::default()
    }

    fn handle_selector_hover(&mut self, pos: &aurora_core::geometry::point::Point, rect: &Rect) -> EventResponse {
        let grid_rect = self.selector_grid_rect(rect);
        self.hovered_selector_month = None;
        self.hovered_selector_year = None;

        let show_months = matches!(self.selector_view, SelectorView::MonthColumn | SelectorView::Combined);
        let show_years = matches!(self.selector_view, SelectorView::YearColumn | SelectorView::Combined);

        if show_months && show_years {
            let half_w = grid_rect.width() / 2.0;
            let month_col = Rect::new(grid_rect.x1, grid_rect.y1, grid_rect.x1 + half_w, grid_rect.y2);
            let year_col = Rect::new(grid_rect.x1 + half_w, grid_rect.y1, grid_rect.x2, grid_rect.y2);
            self.hovered_selector_month = self.selector_month_at(pos, &month_col);
            self.hovered_selector_year = self.selector_year_at(pos, &year_col);
        } else if show_months {
            self.hovered_selector_month = self.selector_month_at(pos, &grid_rect);
        } else if show_years {
            self.hovered_selector_year = self.selector_year_at(pos, &grid_rect);
        }

        let hovering = self.hovered_selector_month.is_some() || self.hovered_selector_year.is_some();
        EventResponse {
            status: EventStatus::Consumed,
            cursor: Some(if hovering { CursorIcon::Pointer } else { CursorIcon::Default }),
            ..Default::default()
        }
    }

    // ── Accessors for CalendarRange ─────────────────────────────────────

    pub(crate) fn header_height_val(&self) -> f32 { self.header_height }
    pub(crate) fn cell_size_val(&self) -> f32 { self.cell_size }
    pub(crate) fn day_layouts(&self) -> &[Option<aurora_text::text_layout::TextLayout>] { &self.day_layouts }
    pub(crate) fn selected_fg_val(&self) -> Color { self.selected_fg }
}
